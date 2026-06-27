use std::fmt;

/*
C++:
    class Player {
    public:
        virtual const std::string& name() const = 0;  // 순수 가상 함수
        virtual int hp() const = 0;
        virtual int attack_power() const = 0;
        virtual bool is_alive() const { return hp() > 0; } // 기본 구현
        virtual ~Player() = default;
    };

Rust: trait 키워드로 인터페이스를 정의
    - 본문이 없는 함수 -> 구현 강제 (C++ 순수 가상 함수와 동일)
    - 본문이 있는 함수 -> 기본 구현 제공, 필요하면 각 타입에서 재정의 가능
*/

trait Player {
    // 구현이 강제되는 함수들 (C++ 순수 가상 함수에 대응)
    fn name(&self) -> &str;
    fn hp(&self) -> i32;
    fn attack_power(&self) -> i32;

    // 기본 구현이 있는 함수 (재정의 하지 않으면 이 코드가 그대로 사용됨)
    fn is_alive(&self) -> bool {
        self.hp() > 0
    }

    fn description(&self) -> String {
        format!(
            "[{}] HP: {} | 공격력: {}",
            self.name(),
            self.hp(),
            self.attack_power()
        )
    }
}

struct Warrior {
    name: String,
    hp: i32,
    strength: i32, // 힘 스탯 -> 공격력 계산에 사용
}

impl Warrior {
    fn new(name: &str, hp: i32, strength: i32) -> Self {
        Warrior {
            name: name.to_string(),
            hp,
            strength,
        }
    }
}

// C++: class Warrior : public Player { ... };
// Rust: impl Trait for Struct  (상속 없이 Trait을 구현)
impl Player for Warrior {
    fn name(&self) -> &str {
        &self.name
    }

    fn hp(&self) -> i32 {
        self.hp
    }

    fn attack_power(&self) -> i32 {
        self.strength * 2 // 전사는 힘이 공격력의 2배
    }

    // description() 은 재정의하지 않음 -> Trait의 기본 구현 사용
}

struct Mage {
    name: String,
    hp: i32,
    mp: i32,    // 마나
    magic: i32, // 마력
}

impl Mage {
    fn new(name: &str, hp: i32, mp: i32, magic: i32) -> Self {
        Mage {
            name: name.to_string(),
            hp,
            mp,
            magic,
        }
    }
}

impl Player for Mage {
    fn name(&self) -> &str {
        &self.name
    }

    fn hp(&self) -> i32 {
        self.hp
    }

    fn attack_power(&self) -> i32 {
        if self.mp > 0 {
            self.magic * 3 // 마나가 있으면 강력한 마법 공격!
        } else {
            5 // 마나 없으면 맨손
        }
    }

    // description() 재정의 -> MP 정보도 함께 표시
    fn description(&self) -> String {
        format!(
            "[{}] HP: {} | MP: {} | 공격력: {}",
            self.name(),
            self.hp(),
            self.mp,
            self.attack_power()
        )
    }
}

struct Archer {
    name: String,
    hp: i32,
    agility: i32, // 민첩
    arrows: i32,  // 화살 수
}

impl Archer {
    fn new(name: &str, hp: i32, agility: i32, arrows: i32) -> Self {
        Archer {
            name: name.to_string(),
            hp,
            agility,
            arrows,
        }
    }
}

impl Player for Archer {
    fn name(&self) -> &str {
        &self.name
    }

    fn hp(&self) -> i32 {
        self.hp
    }

    fn attack_power(&self) -> i32 {
        if self.arrows > 0 {
            self.agility + 10 // 화살이 있으면 민첩 + 10
        } else {
            self.agility / 2 // 화살이 없으면 근접전
        }
    }

    fn description(&self) -> String {
        format!(
            "[{}] HP: {} | 화살: {}개 | 공격력: {}",
            self.name(),
            self.hp(),
            self.arrows,
            self.attack_power()
        )
    }
}

/*
std::fmt::Display Trait 구현
C++의 operator<< 오버로딩에 대응

C++: std::ostream& operator<<(std::ostream& os, const Warrior& w) { ... }
Rust: impl std::fmt::Display for Warrior { fn fmt(...) }
    -> 구현하면 println!("{}", warrior) 처럼 출력 가능
*/

impl fmt::Display for Warrior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "전사 {}", self.description())
    }
}

impl fmt::Display for Mage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "마법사 {}", self.description())
    }
}

impl fmt::Display for Archer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "궁수 {}", self.description())
    }
}

/*
Trait을 함수 인자로 받는 두 가지 방법

방법 1: impl Trait -> 정적 디스패치 (컴파일 타임에 타입이 결정)
C++의 템플릿과 유사: template<typename T> void show_status(const T& c)
컴파일러가 Warrior용, Mage용, Archer용 함수를 각각 별도로 생성함 (monomorphization)
    -> 장점: 런타임 오버헤드 없음
    -> 단점: 런타임에 "이 타입 아니면 저 타입"으로 분기하는 것이 불가능
*/
fn show_status(player: &impl Player) {
    println!("  {}", player.description());
}

/*
방법 2: dyn Trait -> 동적 디스패치 (런타임에 타입이 결정)

C++의 가상 함수 포인터(vtable)와 동일한 원리
런타임에 vtable을 조회해서 실제 타입의 함수를 호출함
    -> 장점: 런타임에 다양한 타입을 동일하게 처리 가능
    -> 단점: vtable 조회 비용 발생 (일반적으로 무시 가능한 수준)

참고: vtable 은 Virtual Function Table 의 약자, 함수와 그에 관련된 정보를 담은 데이터 구조
*/
fn announce_battle(a: &dyn Player, b: &dyn Player) {
    println!("\n=== {} vs {} ===", a.name(), b.name());
    println!("  도전자 -> {}", a.description());
    println!("  방어자 -> {}", b.description());
} 

/*
Box<dyn Trait>: 서로 다른 타입을 하나의 Vec에 담기

C++: std::vector<std::unique_ptr<Player>>
Rust: Vec<Box<dyn Player>>

Box<T> = C++의 unique_ptr<T> (힙 할당, 소유권 단독 보유)
Box<dyn Player> = 힙에 올린 "Player Trait을 구현한 어떤 타입"
*/
fn create_party() -> Vec<Box<dyn Player>> {
    vec![
        Box::new(Warrior::new("홍길동", 120, 25)),
        Box::new(Mage::new("장영실", 70, 80, 30)),
        Box::new(Archer::new("이순신", 90, 20, 15)),
    ]
}

fn show_party(party: &[Box<dyn Player>]) {
    println!("=== 파티 현황 ===");

    for member in party {
        // member 는 &Box<dyn Player>
        // Rust가 자동으로 역참조(deref)하여 dyn Player 의 메서드를 호출
        println!("  {}", member.description());
    }
}

fn main() {
    println!("=== 테스트 1: Display Trait (operator<< 오버로딩) ===");

    let warrior = Warrior::new("홍길동", 120, 25);
    let mage = Mage::new("장영실", 70, 80, 30);
    let archer = Archer::new("이순신", 90, 20, 15);

    // Display Trait 구현 덕분에 println!("{}", ...) 가능
    // C++의 std::cout << warrior << std::endl; 과 동일
    println!("{}", warrior);
    println!("{}", mage);
    println!("{}", archer);

    // ==========================================

    println!("\n=== 테스트 2: impl Trait — 정적 디스패치 ===");

    // 컴파일러가 각 타입별로 show_status 함수를 따로 만들어냄
    // -> 실행 속도는 가장 빠름
    show_status(&warrior); // Warrior 버전으로 컴파일
    show_status(&mage); // Mage 버전으로 컴파일
    show_status(&archer); // Archer 버전으로 컴파일

    // ==========================================

    println!("\n=== 테스트 3: dyn Trait — 동적 디스패치 ===");

    // 런타임에 vtable을 통해 실제 타입의 함수를 찾아서 호출
    // -> 호출하는 쪽 코드는 하나지만, 타입에 따라 다른 동작
    announce_battle(&warrior, &mage);
    announce_battle(&mage, &archer);

    // ==========================================

    println!("\n=== 테스트 4: Box<dyn Trait> — 서로 다른 타입을 하나의 Vec에 ===");

    // Warrior, Mage, Archer 는 모두 다른 타입이지만
    // Player Trait을 구현했으므로 Vec<Box<dyn Player>> 에 함께 담을 수 있음
    // C++: std::vector<std::unique_ptr<Player>>
    let party = create_party();
    show_party(&party);

    // ==========================================

    println!("\n=== 테스트 5: Trait 기본 구현 — is_alive() ===");

    // is_alive() 는 Warrior/Mage/Archer 어디에도 재정의하지 않음
    // -> Trait 에 정의된 기본 구현이 모든 타입에 공통 적용
    println!("생존 여부:");
    for member in &party {
        println!(
            "  {} -> {}",
            member.name(),
            if member.is_alive() {
                "생존"
            } else {
                "사망"
            }
        );
    }
}

/*
🤔 생각해보기:

C++:
1. class Player { virtual void attack() const = 0; }; (순수 가상 함수 -> 구현 강제)
2. class Warrior : public Player { ... };             (상속으로 구현)
3. template<typename T> void fn(const T& c)           (정적 디스패치, 템플릿)
4. void fn(const Player* c)                           (동적 디스패치, vtable)
5. std::vector<std::unique_ptr<Player>>               (다형성 컬렉션)
6. std::ostream& operator<<(std::ostream&, const T&)  (출력 오버로딩)

Rust:
1. trait Player { fn attack(&self); }                 (본문 없음 -> 구현 강제)
2. impl Player for Warrior { ... }                    (상속 없이 Trait 구현)
3. fn show(c: &impl Player)                           (정적 디스패치, monomorphization)
4. fn show(c: &dyn Player)                            (동적 디스패치, vtable)
5. Vec<Box<dyn Player>>                               (다형성 컬렉션)
6. impl std::fmt::Display for Warrior { fn fmt(...) } (출력 오버로딩)

핵심 개념 (Rust Trait의 특징):
    1. Trait = C++ 순수 가상 함수 인터페이스, 단 상속 트리 없이 훨씬 유연하게 붙일 수 있음
    2. Trait 기본 구현 = C++에서 순수 가상 함수는 불가능했지만, Rust Trait은 기본 구현 제공 가능
    3. impl Trait  -> 컴파일 타임 결정, 빠름 (C++ 템플릿과 동일 원리)
    4. dyn Trait   -> 런타임 결정, vtable 사용 (C++ virtual과 동일 원리)
    5. Box<dyn T>  -> C++의 unique_ptr<Interface>와 동일
    6. Display     -> C++의 operator<< 오버로딩과 동일

    -> Rust에는 상속이 없다! 대신 Trait 조합으로 모든 다형성을 표현한다.
        기존 타입에 나중에 Trait을 추가하는 것도 가능 (C++에선 불가능!)
*/