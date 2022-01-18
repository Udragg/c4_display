use c4_display::{DisplayInterface, LedColor, PinConfig, Rotation, Running, Stopped, SyncType};

const W: usize = 4;
const H: usize = 4;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let mut disp: DisplayInterface<Running, 4, 4> = DisplayInterface::<Stopped, W, H>::new("id")
        .start(
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

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        match input.trim().to_lowercase().as_str() {
            "stop" | "s" | "quit" | "q" | "exit" | "e" => {
                disp.stop();
                break;
            }
            "red" | "r" => disp
                .sync(SyncType::All(vec![vec![LedColor::Red; W]; H]))
                .unwrap(),
            "green" | "g" => disp
                .sync(SyncType::All(vec![vec![LedColor::Green; W]; H]))
                .unwrap(),
            "blue" | "b" => disp
                .sync(SyncType::All(vec![vec![LedColor::Blue; W]; H]))
                .unwrap(),
            "yellow" | "y" => disp
                .sync(SyncType::All(vec![vec![LedColor::Yellow; W]; H]))
                .unwrap(),
            "magenta" | "m" => disp
                .sync(SyncType::All(vec![vec![LedColor::Magenta; W]; H]))
                .unwrap(),
            "cyan" | "c" => disp
                .sync(SyncType::All(vec![vec![LedColor::Cyan; W]; H]))
                .unwrap(),
            "white" | "w" => disp
                .sync(SyncType::All(vec![vec![LedColor::White; W]; H]))
                .unwrap(),
            "off" | "o" => disp
                .sync(SyncType::All(vec![vec![LedColor::Off; W]; H]))
                .unwrap(),
            "left" | "counterclockwise" | "cc" => disp
                .sync(SyncType::Rotate(Rotation::CounterClockwise))
                .unwrap(),
            "right" | "clockwise" | "cw" => {
                disp.sync(SyncType::Rotate(Rotation::Clockwise)).unwrap()
            }
            "180" => disp.sync(SyncType::Rotate(Rotation::OneEighty)).unwrap(),
            "custom" => disp
                .sync(SyncType::All(vec![
                    vec![
                        LedColor::Green,
                        LedColor::Blue,
                        LedColor::Blue,
                        LedColor::Blue,
                    ],
                    vec![
                        LedColor::Green,
                        LedColor::Blue,
                        LedColor::White,
                        LedColor::White,
                    ],
                    vec![
                        LedColor::Green,
                        LedColor::Green,
                        LedColor::Red,
                        LedColor::White,
                    ],
                    vec![LedColor::Red, LedColor::Red, LedColor::Red, LedColor::White],
                ]))
                .unwrap(),
            _ => println!("Invalid: {}", input.trim()),
        }
    }
}
