use std::fmt;
use std::num::ParseIntError;

/*
C++:
    void buy_item(Player& p, const std::string& name) {
        if (p.gold < price) {
            throw NotEnoughGoldException(price, p.gold); // 예외 던지기
        }
        ...
    }

    try {
        buy_item(player, "미스릴 갑옷");
    } catch (NotEnoughGoldException& e) {
        std::cerr << e.what();
    } catch (ItemNotFoundException& e) {
        std::cerr << e.what();
    }

문제점: 함수 시그니처만 봐서는 buy_item이 예외를 던지는지 알 수 없다. (noexcept가 없는 한) 실수로 catch를 빼먹어도 컴파일은 통과된다.

Rust:
    fn buy_item(p: &mut Player, name: &str) -> Result<(), GameError>

반환 타입 자체에 "실패할 수 있음"이 명시됨. Result를 무시하면 컴파일러가 경고를 준다 (#[must_use]).
예외처럼 스택을 타고 "어딘가에서" 잡히는 게 아니라, 호출한 쪽이 명시적으로 처리하거나 ? 연산자로 다시 위임해야 한다.
*/

// 커스텀 에러 타입 정의
#[derive(Debug)]
enum GameError {
    NotEnoughGold { needed: i32, have: i32 },
    ItemNotFound(String),
    LevelTooLow { required: u32, current: u32 },
    InvalidInput(String),
}

// ex02에서 배운 Display Trait을 여기서도 그대로 활용, C++의 what() 오버라이드와 동일한 역할
impl fmt::Display for GameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameError::NotEnoughGold { needed, have } => {
                write!(f, "골드가 부족합니다 (필요: {}, 보유: {})", needed, have)
            }
            GameError::ItemNotFound(name) => {
                write!(f, "'{}' 아이템을 상점에서 찾을 수 없습니다", name)
            }
            GameError::LevelTooLow { required, current } => {
                write!(f, "레벨이 부족합니다 (필요: {}, 현재: {})", required, current)
            }
            GameError::InvalidInput(msg) => {
                write!(f, "잘못된 입력입니다: {}", msg)
            }
        }
    }
}

/*
// From Trait: Rust 표준 라이브러리에 준비된 Trait. 서로 다른 에러 타입을 자동으로 변환
// C++: catch (...) 로 뭉뚱그리거나, 각 예외 타입을 일일이 잡아 재포장해야 함
// Rust: From<T>를 구현해두면 ? 연산자가 "알아서" 타입 변환을 해준다.
*/
// 아래 구현 덕분에, ParseIntError가 발생하는 지점에서 ? 를 쓰면 자동으로 GameError::InvalidInput 으로 감싸져서 전파된다.
impl From<ParseIntError> for GameError {
    fn from(e: ParseIntError) -> Self {
        GameError::InvalidInput(e.to_string())
    }
}

struct Player {
    name: String,
    gold: i32,
    level: u32,
    inventory: Vec<String>,
}

struct Item {
    name: String,
    price: i32,
    required_level: u32,
}

/*
아이템 검색: Option 과 Result 의 차이점, 그리고 Result 를 반환하는 이유
Option<T> : "값이 있을 수도, 없을 수도" (이유는 중요하지 않음, 임의)
Result<T, E> : "성공하거나, 실패하면 왜 실패했는지 알아야 함" (결과)
*/
// 상점에서 아이템을 못 찾은 이유를 호출자에게 알려주고 싶으므로 Result 사용.
fn find_item<'a>(shop: &'a [Item], name: &str) -> Result<&'a Item, GameError> {
    // .find()는 Option<&Item>을 반환 -> ok_or_else로 Result로 변환
    // Rust 의 표준 함수는 . 을 이용한 "닷 체이닝" 형 함수가 굉장히 많고 관례적으로 사용됨.
    shop.iter()
        .find(|item| item.name == name)
        .ok_or_else(|| GameError::ItemNotFound(name.to_string()))
}

/*
? 연산자: 에러를 만나면 즉시 반환

C++ (예외 없이 에러 코드로 처리한다면):
    auto item = find_item(shop, name);
    if (!item.has_value()) { return item.error(); }  // 매번 이렇게 손으로 체크

Rust: 위 패턴을 ? 한 글자로 대체
    let item = find_item(shop, name)?; // Err면 즉시 함수에서 return Err(...), Ok면 안의 값만 꺼내서 계속 진행
*/
fn buy_item(player: &mut Player, shop: &[Item], name: &str) -> Result<(), GameError> {
    let item = find_item(shop, name)?; // 못 찾으면 여기서 즉시 Err 반환하고 함수 종료

    if player.level < item.required_level {
        return Err(GameError::LevelTooLow {
            required: item.required_level,
            current: player.level,
        });
    }

    if player.gold < item.price {
        return Err(GameError::NotEnoughGold {
            needed: item.price,
            have: player.gold,
        });
    }

    player.gold -= item.price;
    player.inventory.push(item.name.clone());

    Ok(())
}

// ParseIntError가 발생할 수 있는 지점에서 ? 사용 -> From 변환이 자동으로 일어남
fn parse_level_input(input: &str) -> Result<u32, GameError> {
    let level: u32 = input.trim().parse()?; // 실패 시 ParseIntError -> GameError로 자동 변환되어 전파

    Ok(level)
}

fn level_up(player: &mut Player, input: &str) -> Result<(), GameError> {
    let add = parse_level_input(input)?; // 여기서도 에러면 즉시 상위로 전파

    player.level += add;

    println!(
        "  {}의 레벨이 {} 상승! (현재 레벨: {})",
        player.name, add, player.level
    );

    Ok(())
}

// 이 함수 안에서 세 번의 실패 지점이 있지만, 각각 if-else로 체크하지 않고
// ? 로 흘려보내기만 하면 된다. C++ 이었다면 매 단계마다 try-catch가 필요했을 코드.
fn process_purchase(player: &mut Player, shop: &[Item], item_name: &str, level_input: &str) -> Result<(), GameError> {
    level_up(player, level_input)?; // 1단계: 레벨업 시도
    buy_item(player, shop, item_name)?; // 2단계: 구매 시도

    println!("  -> 구매 절차가 모두 정상적으로 완료되었습니다.");

    Ok(())
}

fn main() {
    let shop = vec![
        Item {
            name: "낡은 검".to_string(),
            price: 50,
            required_level: 1,
        },
        Item {
            name: "미스릴 갑옷".to_string(),
            price: 500,
            required_level: 10,
        },
        Item {
            name: "전설의 활".to_string(),
            price: 1000,
            required_level: 20,
        },
    ];

    let mut player = Player {
        name: "홍길동".to_string(),
        gold: 100,
        level: 1,
        inventory: vec![],
    };

    println!("=== 테스트 1: 정상 구매 ===");

    // match 로 Result 를 직접 처리 - C++의 try/catch 블록에 대응
    match buy_item(&mut player, &shop, "낡은 검") {
        Ok(()) => println!("  구매 성공! 인벤토리: {:?}", player.inventory),
        Err(e) => println!("  구매 실패: {}", e), // Display Trait 덕분에 {} 로 출력 가능
    }

    println!("\n=== 테스트 2: 존재하지 않는 아이템 ===");

    match buy_item(&mut player, &shop, "엑스칼리버") {
        Ok(()) => println!("  구매 성공!"),
        Err(e) => println!("  구매 실패: {}", e),
    }

    println!("\n=== 테스트 3: 레벨 부족 ===");

    match buy_item(&mut player, &shop, "미스릴 갑옷") {
        Ok(()) => println!("  구매 성공!"),
        Err(e) => println!("  구매 실패: {}", e),
    }

    println!("\n=== 테스트 4: 골드 부족 ===");

    player.level = 20; // 레벨 조건은 통과시켜서 골드 부족만 확인

    match buy_item(&mut player, &shop, "전설의 활") {
        Ok(()) => println!("  구매 성공!"),
        Err(e) => println!("  구매 실패: {}", e),
    }

    println!("\n=== 테스트 5: parse() 실패와 From 자동 변환 ===");

    match parse_level_input("5") {
        Ok(n) => println!("  파싱 성공: {}", n),
        Err(e) => println!("  파싱 실패: {}", e),
    }

    // "다섯" -> 숫자가 아니기에 ParseIntError 가 발생하지만, From<ParseIntError> 덕분에 GameError::InvalidInput 으로 자동 변환되어 도착함
    match parse_level_input("다섯") {
        Ok(n) => println!("  파싱 성공: {}", n),
        Err(e) => println!("  파싱 실패: {}", e),
    }

    println!("\n=== 테스트 6: ? 연산자로 여러 단계 연결하기 ===");

    player.gold = 1000; // 구매 가능하도록 초기화

    // 정상 흐름: 레벨업도 성공, 구매도 성공
    match process_purchase(&mut player, &shop, "낡은 검", "3") {
        Ok(()) => println!("  전체 프로세스 성공"),
        Err(e) => println!("  프로세스 중단: {}", e),
    }

    // 중간에 실패: 레벨업 입력값이 잘못됨 -> buy_item은 실행조차 안 됨
    match process_purchase(&mut player, &shop, "낡은 검", "삼") {
        Ok(()) => println!("  전체 프로세스 성공"),
        Err(e) => println!("  프로세스 중단: {}", e), // level_up 단계에서 ? 로 즉시 빠져나옴
    }
}

/*
🤔 생각해보기:

C++:
    1. throw MyException(...)                        (예외 던지기, 스택을 타고 전파)
    2. try { ... } catch (MyException& e) {...}      (호출부에서 명시적으로 잡아야 함, 안 잡아도 컴파일은 됨)
    3. catch (...)                                   (모든 예외를 뭉뚱그려 잡기)
    4. 함수 시그니처만으론 예외 발생 여부를 알 수 없음
    5. 예외 재포장: catch한 뒤 다른 타입으로 다시 throw

Rust:
    1. return Err(GameError::...)                  (그냥 평범한 값을 반환할 뿐, 스택 언와인딩 없음)
    2. match result { Ok(v) => ..., Err(e) => ...} (반드시 처리해야 컴파일 경고가 없음, #[must_use])
    3. Err(_) => { ... }                           (와일드카드로 모든 에러 케이스 처리 가능)
    4. fn f(...) -> Result<T, GameError>           (시그니처에 실패 가능성이 항상 명시됨)
    5. impl From<OtherError> for GameError         (? 연산자가 자동으로 타입 변환하며 전파)

핵심 개념:
    1. Result<T, E> 는 예외가 아니라 그냥 값이다 (Ok(T) 아니면 Err(E))
    2. ? 연산자 = "에러면 즉시 Err로 함수 반환, 아니면 값을 꺼내서 계속 진행"
    3. enum 하나로 여러 에러 케이스를 표현하는 것이 Rust의 관례
    4. From<E> 구현 -> ? 가 서로 다른 에러 타입 간 자동 변환까지 처리해줌
    5. Result를 무시하면 컴파일러가 경고 (예외처럼 "빼먹어도 일단 컴파일되는" 상황이 없음)
*/
