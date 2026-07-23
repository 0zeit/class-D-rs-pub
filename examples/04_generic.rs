/*
C++:
    template<typename T>
    class Inventory {
        std::vector<T> items;
    public:
        void add(T item) { items.push_back(item); }
        size_t count() const { return items.size(); }
    };

    Inventory<Weapon> weapons;
    Inventory<Potion> potions;

Rust: 문법은 다르지만 개념은 똑같다. <T> 라는 "타입을 담는 매개변수"를 선언해두면,
    실제로 쓰이는 시점에 컴파일러가 T 자리에 구체적인 타입(Weapon, Potion, i32...)을 채워 넣는다.

    struct Inventory<T> {
        items: Vec<T>,
    }

- 사실 여러분은 이미 제네릭을 계속 써왔다.
    - ex01의 Option<Weapon>          -> Option<T> 의 T 자리에 Weapon이 들어간 것
    - ex03의 Result<u32, GameError>  -> Result<T, E> 의 T, E 자리에 각각 들어간 것
    - ex03의 Vec<Item>               -> Vec<T> 의 T 자리에 Item이 들어간 것

이번 예제에서는 이 <T> 를 "직접 선언"하는 법을 배운다.
*/

// Valuable: "가치를 매길 수 있는 것" 이라는 조건을 표현하는 Trait (ex02에서 배운 개념 재활용)
trait Valuable {
    fn value(&self) -> i32;
    fn label(&self) -> &str;
}

struct Weapon {
    name: String,
    damage: i32,
}

impl Valuable for Weapon {
    fn value(&self) -> i32 {
        self.damage
    }

    fn label(&self) -> &str {
        &self.name
    }
}

struct Potion {
    name: String,
    heal: i32,
}

impl Valuable for Potion {
    fn value(&self) -> i32 {
        self.heal
    }

    fn label(&self) -> &str {
        &self.name
    }
}

// 제네릭 구조체: T 자리에 어떤 타입이 올지는 "쓰이는 시점"에 결정됨
struct Inventory<T> {
    items: Vec<T>,
}

// 조건 없는 impl: T가 무엇이든 상관없이 항상 쓸 수 있는 메서드
impl<T> Inventory<T> {
    fn new() -> Self {
        Inventory { items: Vec::new() }
    }

    fn add(&mut self, item: T) {
        self.items.push(item);
    }

    fn count(&self) -> usize {
        self.items.len()
    }
}

/*
조건부 impl: T: Valuable 이라는 "특성 제약(Trait Bound)"이 있어야만 컴파일되는 블록

C++20 이전: 
    template<typename T> T largest(...) { return a > b ? a : b; }
    -> T가 > 연산자를 지원하지 않으면, 에러가 "템플릿이 실제로 쓰이는 지점"에서야 뒤늦게 발생 (길고 난해한 에러)
C++20 이후: 
    template<typename T> requires std::totally_ordered<T> T largest(...)
    -> concept으로 조건을 명시하면, 조건에 안 맞는 타입은 훨씬 이른 시점에 명확한 에러로 걸러짐 (concept는 아직 배우지 않음!)

Rust는 처음부터 이 방식이었다. <T: Valuable> 이 곧 C++20의 concept 제약과 같은 역할을 한다.
Inventory<T>에 담긴 T가 Valuable을 구현하지 않으면, 아래 메서드들은 애초에 "존재하지 않는 것"처럼 취급된다.
*/
impl<T: Valuable> Inventory<T> {
    fn total_value(&self) -> i32 {
        self.items.iter().map(|item| item.value()).sum()
    }

    // .max_by_key() 는 Option<&T> 를 반환 -> 비어있으면 None (ex01의 Option과 같은 발상)
    fn strongest(&self) -> Option<&T> {
        self.items.iter().max_by_key(|item| item.value())
    }
}

/*
독립 제네릭 함수 + PartialOrd 특성 제약(Trait Bound)
PartialOrd 라는 표준 라이브러리가 제공하는 특성으로, 이 함수는 그에 대한 제약이 걸린 함수. ">" "<" 같은 비교 연산자를 지원한다는 뜻.

C++:
    template<typename T> T largest(const std::vector<T>& list) { ... a > b ... }
Rust:
    fn largest<T: PartialOrd>(list: &[T]) -> &T

이 바운드가 없으면 *item > *result 같은 코드가 "T가 비교 가능하다는 보장이 없다"는 이유로 컴파일 자체가 거부된다.
*/
fn largest<T: PartialOrd>(list: &[T]) -> &T {
    let mut result = &list[0];

    for item in list {
        if *item > *result { // 여기가 막히게 됨
            result = item;
        }
    }

    result
}

fn main() {
    println!("=== 테스트 1: 기본 제네릭 구조체 — Inventory<T> ===");

    let mut weapon_box: Inventory<Weapon> = Inventory::new();

    weapon_box.add(Weapon { name: "낡은 검".to_string(), damage: 12 });
    weapon_box.add(Weapon { name: "강철검".to_string(), damage: 30 });
    weapon_box.add(Weapon { name: "미스릴 대검".to_string(), damage: 55 });

    println!("  무기 인벤토리 개수: {}", weapon_box.count());

    println!("\n=== 테스트 2: 조건부 impl — Trait Bound가 있어야 쓸 수 있는 메서드 ===");

    // T(Weapon)가 Valuable을 구현하므로, impl<T: Valuable> 블록의 메서드 사용 가능
    println!("  무기 전체 가치 합: {}", weapon_box.total_value());

    match weapon_box.strongest() {
        Some(w) => println!("  가장 강한 무기: {} (공격력 {})", w.label(), w.value()),
        None => println!("  인벤토리가 비어있습니다"),
    }

    // 완전히 같은 Inventory<T> 코드가 T = Potion 일 때도 동일하게 작동함 (C++ 템플릿처럼, 코드는 한 번만 작성하고 컴파일러가 각 타입별로 찍어냄)
    let mut potion_box: Inventory<Potion> = Inventory::new();

    potion_box.add(Potion { name: "하급 물약".to_string(), heal: 20 });
    potion_box.add(Potion { name: "상급 물약".to_string(), heal: 80 });

    println!("  물약 전체 회복량 합: {}", potion_box.total_value());

    match potion_box.strongest() {
        Some(p) => println!("  가장 강한 물약: {} (회복량 {})", p.label(), p.value()),
        None => println!("  인벤토리가 비어있습니다"),
    }

    println!("\n=== 테스트 3: 독립 제네릭 함수 — largest<T: PartialOrd> ===");

    let levels = vec![12, 45, 7, 88, 33];
    let scores = vec![3.5, 9.1, 4.4, 7.8];

    // 같은 largest() 함수가 i32에도, f64에도 그대로 재사용됨
    println!("  최고 레벨: {}", largest(&levels));
    println!("  최고 점수: {}", largest(&scores));

    println!("\n=== 테스트 4: 동종 컬렉션 vs 이종 컬렉션 ===");

    println!("  Inventory<Weapon>은 Weapon만, Inventory<Potion>은 Potion만 담을 수 있음");
    println!("  (T는 한 번 정해지면 그 Inventory 안에서는 고정됨)");

    // weapon_box.add(Potion { name: "물약".to_string(), heal: 10 }); <- 컴파일 에러! expected `Weapon`, found `Potion`
    // ex02의 Vec<Box<dyn Player>>는 서로 다른 타입(Warrior, Mage, Archer)을 한 Vec에 담을 수 있었지만,
    // Inventory<T>의 T는 "딱 하나의 구체적인 타입"으로 컴파일 시점에 고정된다. -> 제네릭(<T>)은 "동종" 컬렉션, dyn Trait은 "이종" 컬렉션에 어울린다.

    /* 예제 02 에서의 코드, Vec<Box<dyn Player>> 에 주목! (이종 컬렉션)
    let party: Vec<Box<dyn Player>> = vec![
        Box::new(Warrior::new(...)), // Warrior 타입
        Box::new(Mage::new(...)),    // Mage 타입 (다른 타입인데 같은 Vec에!)
        Box::new(Archer::new(...)),  // Archer 타입
    ];
    */

    println!("\n=== 테스트 5: 터보피시(turbofish) 문법 ===");

    // 보통은 타입 추론에 맡기지만, 애매할 때는 ::<> 로 직접 타입을 지정할 수 있다.
    // ::<> 의 명칭 자체가 물고기 모양을 닮아 "터보피시(turbofish)" 라는 이름이 붙어있다.
    let inferred: Inventory<Weapon> = Inventory::new();            // 방법 1: 변수 타입 명시
    let turbofish = Inventory::<Weapon>::new(); // 방법 2: 터보피시로 직접 명시

    println!("  변수 타입 명시로 생성: 개수 {}", inferred.count());
    println!("  터보피시로 생성: 개수 {}", turbofish.count());

    // parse() 도 자주 터보피시와 함께 쓰인다 (ex03에서는 변수 타입 명시로 처리했었음)
    let n = "42".parse::<i32>().unwrap();

    println!("  \"42\".parse::<i32>() 결과: {}", n);
}

/*
🤔 생각해보기:

C++:
    1. template<typename T> class Inventory { ... };         (제네릭 클래스)
    2. template<typename T> T largest(...)                   (제약 없는 제네릭 함수 - 에러가 늦게 발생)
    3. template<typename T> requires std::totally_ordered<T> (C++20 concept - 제약을 명시)
    4. std::vector<T>                                        (표준 라이브러리도 이미 제네릭)
    5. std::vector<std::unique_ptr<Interface>>               (이종 컬렉션은 다형성으로 별도 처리)

Rust:
    1. struct Inventory<T> { items: Vec<T> }        (제네릭 구조체)
    2. fn largest<T>(list: &[T]) -> &T              (바운드 없이는 비교 연산 자체가 불가능 - 컴파일 자체가 안 됨)
    3. fn largest<T: PartialOrd>(list: &[T]) -> &T  (Trait Bound = C++20 concept과 동일한 역할)
    4. Option<T>, Result<T, E>, Vec<T>              (이미 계속 써온 표준 제네릭들)
    5. Vec<Box<dyn Trait>>                          (이종 컬렉션, ex02에서 다룸)

핵심 개념:
    1. <T> 는 "실제 타입은 나중에 정해진다"는 매개변수 선언 (제네릭 타입 매개변수)
    2. impl<T> 은 무조건, impl<T: Bound> 는 조건부로 메서드를 제공 (컴파일 타임에 검증)
    3. Trait Bound(<T: Valuable>)는 C++20의 concept과 동일한 발상 - "이 타입은 최소한 이런 능력이 있어야 한다"
    4. 제네릭은 컴파일 시 타입별로 코드가 찍혀 나옴 (monomorphization, ex02의 impl Trait과 동일 원리)
        -> 런타임 비용 없음, 대신 하나의 컨테이너 안에는 "한 가지 구체 타입"만 허용됨
    5. 이미 Option<T>, Result<T, E>, Vec<T>를 통해 제네릭을 계속 사용해왔다 - 이제 직접 만들 수 있게 된 것뿐
*/