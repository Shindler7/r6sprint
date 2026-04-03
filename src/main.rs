//! Парсинг лог-файлов.

fn main() {
    println!("Placeholder для экспериментов с cli");

    let parsing_demo =
        r#"[UserBackets{"user_id":"Bob","backets":[Backet{"asset_id":"milk","count":3,},],},]"#
            .to_string();
    let announcements = analysis::parse::just_parse_anouncements(parsing_demo).unwrap();
    println!("demo-parsed: {:?}", announcements);

    let args = std::env::args().collect::<Vec<_>>();
    let filename = args[1].clone();
    println!(
        "Trying opening file '{}' from directory '{}'",
        filename,
        std::env::current_dir().unwrap().to_string_lossy()
    );
    let file: std::rc::Rc<std::cell::RefCell<Box<dyn analysis::MyReader>>> = std::rc::Rc::new(
        std::cell::RefCell::new(Box::new(std::fs::File::open(filename).unwrap())),
    );

    let logs = analysis::read_log(file.clone(), analysis::READ_MODE_ALL, vec![]);
    println!("got logs:");
    logs.iter().for_each(|parsed| println!("  {:?}", parsed));
}
