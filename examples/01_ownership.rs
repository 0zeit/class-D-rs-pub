struct Weapon {
    name: String,
    damage: i32,
}

struct Player {
    name: String,
    hp: i32,
    weapon: Option<Weapon>, // 무기가 없을 수도 있으므로 Option 을 사용! C++에서의 Weapon* weapon = nullptr; 과 같은 역할,
}

impl Player {
    fn new(name: &str, hp: i32) -> Self {
        Player {
            name: name.to_string(),
            hp,
            weapon: None, // 처음엔 무기 없음
        }
    }

    // &mut self : 자기 자신을 "가변 참조"로 받음 (C++의 void equip(Weapon* w) 와 대응)
    fn equip(&mut self, weapon: Weapon) {
        println!("{}이(가) [{}]을 장착!", self.name, weapon.name);

        self.weapon = Some(weapon); // weapon은 여기서 self.weapon 안으로 "이동(Move)"됨. 이 줄 이후로 weapon을 쓰면 컴파일 에러!
    }

    fn unequip(&mut self) -> Option<Weapon> {
        self.weapon.take() // Option에서 꺼내(take) 반환, self.weapon은 None이 됨
    }

    // &self : 자기 자신을 "불변 참조"로 받음 → 읽기 전용! (C++의 void display() const 와 대응)
    fn display(&self) {
        let weapon_info = match &self.weapon {
            Some(w) => format!("[{}] 공격력 {}", w.name, w.damage),
            None => "없음".to_string(),
        };
        println!(
            "  이름: {} | HP: {} | 무기: {}",
            self.name, self.hp, weapon_info
        );
    }

    fn attack_power(&self) -> i32 {
        match &self.weapon {
            Some(w) => w.damage,
            None => 5, // 맨손 공격력
        }
    }
}

struct Enemy {
    name: String,
    hp: i32,
}

impl Enemy {
    fn new(name: &str, hp: i32) -> Self {
        Enemy {
            name: name.to_string(),
            hp,
        }
    }

    fn display(&self) {
        println!("  [{}] HP: {}", self.name, self.hp);
    }
}

// C++:  void battle(Player* player, Enemy* enemy)
// Rust: fn battle(player: &mut Player, enemy: &mut Enemy)
fn battle(player: &mut Player, enemy: &mut Enemy) {
    println!("\n=== 전투 시작: {} vs {} ===", player.name, enemy.name);

    let mut round = 1;

    while 0 < player.hp && 0 < enemy.hp {
        println!("-- 라운드 {} --", round);

        let player_dmg = player.attack_power(); // &self (불변 빌림)
        
        enemy.hp -= player_dmg;
        
        println!("  {} → {} 에게 {} 데미지!", player.name, enemy.name, player_dmg);

        if enemy.hp <= 0 {
            break;
        }

        // 적 반격
        let enemy_dmg = 12;
        
        player.hp -= enemy_dmg;

        println!("  {} → {} 에게 {} 데미지!", enemy.name, player.name, enemy_dmg);

        round += 1;
    }

    println!("=== 전투 종료 ===");

    if 0 < player.hp {
        println!("{}의 승리!\n", player.name);
    } else {
        println!("{}의 패배...\n", player.name);
    }
}


// C++의 std::move + unique_ptr 개념과 동일
fn give_weapon(from: &mut Player, to: &mut Player) {
    match from.unequip() {
        Some(weapon) => {
            println!("\n{}이(가) {}에게 [{}]를 건네줬습니다.", from.name, to.name, weapon.name);

            to.equip(weapon); // 소유권이 'weapon' -> 'to.weapon' 으로 이동
        }
        None => {
            println!("\n{}에게 줄 무기가 없습니다!", from.name);
        }
    }
}

fn main() {
    println!("=== 테스트 1: 소유권 이전 (Move) ===");

    let sword = Weapon {
        name: "강철검".to_string(),
        damage: 30,
    };
    let mut player1 = Player::new("홍길동", 120);
    let mut player2 = Player::new("이순신", 100);

    player1.equip(sword);

    // println!("{}", sword.name);

    println!("[장착 후]");

    player1.display();
    player2.display();

    // ==========================================

    println!("\n=== 테스트 2: 불변 빌림 (&) ===");

    let atk = player1.attack_power(); // &self 빌림
    let power = player1.attack_power(); // 동시에 또 빌려도 OK! (읽기만 하므로)

    player1.display();

    println!("  → 공격력 확인: {} / {}", atk, power);

    // ==========================================

    println!("\n=== 테스트 3: 무기 양도(소유권 이전) ===");

    give_weapon(&mut player1, &mut player2);

    println!("[양도 후]");

    player1.display(); // 무기 없음
    player2.display(); // 강철검 보유

    // ==========================================

    println!("\n=== 테스트 4: 전투(가변 빌림 &mut) ===");

    let mut goblin = Enemy::new("고블린 왕", 70);

    battle(&mut player2, &mut goblin);

    println!("[전투 후]");

    player2.display();
    goblin.display();

    // main() 종료 시 모든 값이 메모리에서 자동 해제(drop), C++의 delete / unique_ptr 불필요!
}

/*
🤔 생각해보기:

C++:
1. Weapon* w = nullptr (nullptr 체크가 누락될 가능성 있음!)
2. std::move(sword)    (Move를 명시적으로 써야함)
3. const Player* p     (읽기 전용 포인터)
4. Player* p           (쓰기 가능 포인터)
5. delete p            (만든 포인터/메모리 해제. 깜빡하면 메모리 누수!)

Rust:
1. weapon: Option<Weapon> (아무것도 없을때 None형 처리를 컴파일러가 강제)
2. player.equip(sword)    (기본적으로 Move를 활용함)
3. player: &Player        (불변 참조, 동시에 여럿 가능)
4. player: &mut Player    (가변 참조, 한번에 하나만 허용)
5. 스코프 종료 시 자동 drop(), 메모리 누수 불가


핵심 규칙 (Rust 컴파일러가 강제함):
  1. 변수의 소유자(Owner)는 항상 하나
  2. &(불변 참조)는 동시에 여러 개의 변수가 참조 가능
  3. &mut(가변 참조)는 동시에 단 하나만의 참조가 가능
  -> 이 세 가지 규칙만으로 C++의 모든 메모리 버그를 컴파일 타임에 차단!
*/
