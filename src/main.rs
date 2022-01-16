use c4_display::{DisplayInterface, LedColor, PinConfig, SyncType};

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut disp = DisplayInterface::<_, 4, 4>::new("id").start(
        60.,
        PinConfig {
            sr_serin: 24,
            sr_srclk: 23,
            sr_rclk: 18,
            sr_srclr: 15,
            sr_oe: 14,
            dec_a0: 6,
            dec_a1: 13,
            dec_a2: 19,
            dec_le: 26,
        },
    );
    // let template = disp.sync_template();

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        match input.trim().to_lowercase().as_str() {
            "stop" | "s" | "quit" | "q" => {
                disp.stop();
                break;
            }
            "red" | "r" => disp
                .sync(SyncType::All(vec![vec![LedColor::Red; 4]; 4]))
                .unwrap(),
            "green" | "g" => disp
                .sync(SyncType::All(vec![vec![LedColor::Green; 4]; 4]))
                .unwrap(),
            "blue" | "b" => disp
                .sync(SyncType::All(vec![vec![LedColor::Blue; 4]; 4]))
                .unwrap(),
            "yellow" | "y" => disp
                .sync(SyncType::All(vec![vec![LedColor::Yellow; 4]; 4]))
                .unwrap(),
            "magenta" | "m" => disp
                .sync(SyncType::All(vec![vec![LedColor::Magenta; 4]; 4]))
                .unwrap(),
            "cyan" | "c" => disp
                .sync(SyncType::All(vec![vec![LedColor::Cyan; 4]; 4]))
                .unwrap(),
            "white" | "w" => disp
                .sync(SyncType::All(vec![vec![LedColor::White; 4]; 4]))
                .unwrap(),
            "off" | "o" => disp
                .sync(SyncType::All(vec![vec![LedColor::Off; 4]; 4]))
                .unwrap(),
            _ => println!("Invalid: {}", input.trim()),
        }
    }
}
