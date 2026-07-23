/*
C++의 동적 디스패치: vtable 포인터가 "객체 안"에 숨겨진 멤버로 저장된다.

    class Skill { public: virtual void cast() = 0; };
    class Fireball : public Skill { public: void cast() override { ... } };

    Fireball f;
    // f 의 메모리 레이아웃: [ vtable 포인터 | power 등 실제 데이터... ]
    // -> virtual 함수가 하나라도 있으면, 그 객체는 항상 vtable 포인터를 짊어지고 다닌다.

Rust의 동적 디스패치: vtable 포인터는 "객체 안"이 아니라 "참조/포인터 쪽"에 저장된다.

    struct Fireball { power: i32 }               // 이 구조체 자체엔 vtable이 없다. 그냥 순수 데이터.
    let f: &dyn Skill = &Fireball { power: 45 }; // &dyn Skill 이라는 "뚱뚱한 포인터(fat pointer)"가 [ 데이터 포인터 | vtable 포인터 ] 두 개를 대신 들고 다닌다.

    -> Fireball을 평범한 구조체로만 쓸 땐 vtable 오버헤드가 전혀 없고, "이걸 dyn Trait으로 다루겠다"고 결정하는 순간에만 참조/Box 쪽에서 추가 비용이 발생한다.
*/

struct Player {
    name: String,
    mana: i32,
}

trait Skill {
    fn name(&self) -> &str;

    fn mana_cost(&self) -> i32 {
        10 // 기본 마나 소모량
    }

    fn cast(&self, caster: &mut Player);
}

struct Fireball {
    power: i32,
}

impl Skill for Fireball {
    fn name(&self) -> &str {
        "파이어볼"
    }

    fn mana_cost(&self) -> i32 {
        30
    }

    fn cast(&self, caster: &mut Player) {
        caster.mana -= self.mana_cost();

        println!("  {}이(가) [파이어볼]을 시전! {} 데미지! (남은 마나: {})", caster.name, self.power, caster.mana);
    }
}

struct Heal {
    amount: i32,
}

impl Skill for Heal {
    fn name(&self) -> &str {
        "힐"
    }

    fn mana_cost(&self) -> i32 {
        20
    }

    fn cast(&self, caster: &mut Player) {
        caster.mana -= self.mana_cost();

        println!("  {}이(가) [힐]을 시전! {} 회복! (남은 마나: {})", caster.name, self.amount, caster.mana);
    }
}

struct Sprint {
    boost: i32,
}

impl Skill for Sprint {
    fn name(&self) -> &str {
        "달리기"
    }

    fn mana_cost(&self) -> i32 {
        5
    }

    fn cast(&self, caster: &mut Player) {
        caster.mana -= self.mana_cost();

        println!("  {}이(가) [달리기]을 시전! 이동속도 +{}! (남은 마나: {})", caster.name, self.boost, caster.mana);
    }
}

/*
객체 안전성(Object Safety): 모든 Trait이 dyn Trait이 될 수 있는 건 아니다.
아래 메서드를 Skill trait에 추가하면 Box<dyn Skill>은 더 이상 컴파일되지 않는다:

fn combo_with<T: Skill>(&self, other: &T) -> String { ... }
// 제네릭 메서드는 호출될 때마다 별도의 코드가 컴파일시 찍혀 나와야 하는데, vtable은 "고정된 크기의 함수 포인터 목록"이라 이런 걸 담을 방법이 없다.

fn clone_self(&self) -> Self { ... }
// Self를 값으로 반환 -> 컴파일러가 반환값의 정확한 크기를 알아야 하는데, dyn Skill 뒤에 실제로 어떤 구체 타입이 있는지는 런타임에만 알 수 있어서 크기를 정할 수 없다.

이런 제약을 만족하는 Trait만 "객체 안전(object-safe)"하다고 부르며, dyn Trait으로 쓸 수 있다.
표준 라이브러리의 Clone이 대표적으로 객체 안전하지 않은 Trait이다 (Self를 반환하기 때문).
*/
struct SkillBook {
    skills: Vec<Box<dyn Skill>>, // 참고: 라이프타임을 생략하면 기본값은 'static (다음 예제에서 다룸)
}

impl SkillBook {
    fn new() -> Self {
        SkillBook { skills: Vec::new() }
    }

    fn learn(&mut self, skill: Box<dyn Skill>) {
        println!("  [{}] 습득!", skill.name());

        self.skills.push(skill);
    }

    fn total_mana_cost(&self) -> i32 {
        self.skills.iter().map(|s| s.mana_cost()).sum()
    }

    fn cast_all(&self, caster: &mut Player) {
        for skill in &self.skills {
            skill.cast(caster); // 동적 디스패치: vtable을 조회해 실제 타입의 cast()를 호출
        }
    }
}

/*
왜 dyn Trait이 "필요"한가: 런타임에야 어떤 타입인지 결정되는 경우

fn create_skill(name: &str) -> impl Skill { // 이렇게 쓰면 컴파일 에러(정적 디스패치)!
    match name {
        "fireball" => Fireball { power: 45 }, // 여기선 Fireball
        "heal" => Heal { amount: 30 },        // 여기선 Heal - 서로 다른 구체 타입!
        _ => Sprint { boost: 0 },
    }
}

impl Trait 반환은 "이 함수는 정확히 한 가지 구체 타입만 반환한다"는 약속이다.
컴파일러는 반환 슬롯의 크기를 컴파일 타임에 하나로 확정해야 하는데,
분기마다 크기도 vtable도 다른 타입을 반환하면 그 약속을 지킬 수 없다.
이럴 땐 "포인터 + vtable"로 크기를 통일해버리는 Box<dyn Skill> 밖에는 방법이 없다.
*/
fn create_skill(name: &str) -> Box<dyn Skill> {
    match name {
        "fireball" => Box::new(Fireball { power: 45 }),
        "heal" => Box::new(Heal { amount: 30 }),
        "sprint" => Box::new(Sprint { boost: 20 }),
        _ => Box::new(Sprint { boost: 0 }), // 알 수 없는 이름 -> 기본값으로 처리
    }
}

/*
클로저도 트레잇 오브젝트가 될 수 있다: Box<dyn Fn(...) -> ...>
클로저(Clousure)에 대해선 다음 시간 알아본다.

Fn, FnMut, FnOnce 도 결국 표준 라이브러리의 Trait일 뿐이다. Fn 은 C++ 의 Functional 과 같은 개념.
그러니 지금까지 본 dyn Trait의 규칙이 클로저에도 그대로 적용된다.
*/
fn apply_buffs(base_damage: i32, buffs: &[Box<dyn Fn(i32) -> i32>]) -> i32 {
    let mut damage = base_damage;

    for buff in buffs {
        damage = buff(damage); // 동적 디스패치로 클로저 호출
    }

    damage
}

fn main() {
    let mut hero = Player {
        name: "홍길동".to_string(),
        mana: 100,
    };

    println!("=== 테스트 1: SkillBook — Vec<Box<dyn Skill>> ===");

    let mut book = SkillBook::new();

    book.learn(Box::new(Fireball { power: 45 }));
    book.learn(Box::new(Heal { amount: 30 }));
    book.learn(Box::new(Sprint { boost: 20 }));

    println!("  전체 마나 소모량: {}", book.total_mana_cost());

    book.cast_all(&mut hero);

    println!("\n=== 테스트 2: 왜 dyn이 '필요'한가 — 런타임 결정 ===");

    // 실제 서비스라면 파일이나 네트워크에서 오는 문자열이라고 상상해보자
    let commands = vec!["fireball", "sprint", "heal", "fireball"];
    let mut runtime_book = SkillBook::new();

    for cmd in &commands {
        runtime_book.learn(create_skill(cmd)); // 문자열마다 다른 구체 타입이 반환됨
    }

    println!("  런타임 커맨드로 구성된 스킬 개수: {}", runtime_book.skills.len());

    println!("\n=== 테스트 3: Fat Pointer — &dyn Trait은 왜 크기가 두 배인가 ===");

    // 아래 수치는 일반적인 64비트 환경 기준
    println!("  size_of::<&Fireball>()      = {} bytes", std::mem::size_of::<&Fireball>());
    println!("  size_of::<&dyn Skill>()     = {} bytes", std::mem::size_of::<&dyn Skill>());
    println!("  size_of::<Box<Fireball>>()  = {} bytes", std::mem::size_of::<Box<Fireball>>());
    println!("  size_of::<Box<dyn Skill>>() = {} bytes", std::mem::size_of::<Box<dyn Skill>>());

    println!("\n=== 테스트 4: 객체 안전성 (Object Safety) ===");

    println!("  Skill trait은 객체 안전함 -> Box<dyn Skill> 가능");
    println!("  (제네릭 메서드나 Self 반환 메서드가 없기 때문)");
    // 위 주석에서 다룬 combo_with<T>(...), clone_self(&self) -> Self 같은 메서드가 있었다면 불가능했음

    println!("\n=== 테스트 5: 클로저도 트레잇 오브젝트다 — Box<dyn Fn> ===");

    let buffs: Vec<Box<dyn Fn(i32) -> i32>> = vec![
        Box::new(|dmg| dmg + 10), // 고정 데미지 증가 버프
        Box::new(|dmg| dmg * 2),  // 데미지 2배 버프
    ];

    let final_damage = apply_buffs(45, &buffs);

    println!("  기본 데미지 45 + 버프 적용 -> 최종 데미지: {}", final_damage);
}

/*
🤔 생각해보기:

C++:
    1. class Skill { virtual void cast() = 0; };   (객체 안에 vtable 포인터가 내장됨)
    2. Skill* / Skill&                             (평범한 포인터, 크기는 그대로)
    3. std::function<int(int)>                     (함수/클로저를 감싸는 타입 지운 래퍼)
    4. std::unique_ptr<Skill>                      (다형성 컬렉션의 원소)
    5. 별도의 "객체 안전성" 규칙이 없음 (사실상 모든 다형 클래스가 포인터로 다뤄질 수 있음)

Rust:
    1. struct Fireball { power: i32 }         (구조체 자체엔 vtable 없음, 순수 데이터)
    2. &dyn Skill / Box<dyn Skill>            (뚱뚱한 포인터 - 데이터 포인터 + vtable 포인터)
    3. Box<dyn Fn(i32) -> i32>                (클로저도 결국 Trait, 동일한 규칙이 그대로 적용)
    4. Vec<Box<dyn Skill>>                    (다형성 컬렉션의 원소)
    5. 객체 안전성(Object Safety) 규칙 존재 - Self 반환/제네릭 메서드가 있으면 dyn 불가

핵심 개념:
    1. dyn Trait의 vtable 포인터는 객체가 아니라 참조/Box 쪽에 저장된다 (fat pointer)
    2. 그래서 평범한 구조체는 dyn 으로 쓰이든 안 쓰이든 자기 자신의 크기가 절대 변하지 않는다
    3. 제네릭(<T>)은 "컴파일 타임에 하나로 고정된 타입"만 다룰 수 있다
       -> 런타임에야 타입이 갈리는 상황(문자열 커맨드, 설정 파일 등)에선 dyn Trait 이 유일한 선택지
    4. 모든 Trait 이 dyn 가능한 건 아니다 - Self 반환, 제네릭 메서드가 있으면 객체 안전하지 않음
    5. Fn/FnMut/FnOnce 도 Trait 이므로, 클로저 역시 Box<dyn Fn(...)> 형태로 동적 디스패치가 가능하다
*/