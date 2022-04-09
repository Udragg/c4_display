use std::str::FromStr;

use c4_display::{
    Animation, DisplayInterface, LedColor, LedState, PinConfig, Rotation, Running, Stopped,
    SyncType,
};

const W: usize = 7;
const H: usize = 7;

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    let mut disp: DisplayInterface<Running, W, H> = DisplayInterface::<Stopped, W, H>::new("id")
        .start(
            60.0,
            PinConfig {
                sr_serin: 17,
                sr_srclk: 22,
                sr_rclk: 23,
                sr_srclr: 24,
                sr_oe: 27,
                dec_a0: 25,
                dec_a1: 11,
                dec_a2: 5,
                dec_le: 6,
                dec_e1: 10,
            },
        );

    println!("started");

    loop {
        let mut input = String::new();
        std::io::stdin().read_line(&mut input).unwrap();
        match input.trim().to_lowercase().as_str() {
            "stop" | "s" | "quit" | "q" | "exit" | "e" => {
                disp.stop();
                break;
            }
            "left" | "counterclockwise" | "cc" => disp
                .sync(SyncType::Rotate(Rotation::CounterClockwise))
                .unwrap(),
            "right" | "clockwise" | "cw" => {
                disp.sync(SyncType::Rotate(Rotation::Clockwise)).unwrap()
            }
            "180" => disp.sync(SyncType::Rotate(Rotation::OneEighty)).unwrap(),
            "circle" => disp
                .add_animation(Animation::from_file("./animations/circle.mtxani").unwrap())
                .unwrap(),
            "ca" => disp.clear_animations(),
            color if LedColor::from_str(color).is_ok() => disp
                .sync(SyncType::All(vec![
                    vec![
                        LedState::with_color(
                            LedColor::from_str(color).unwrap()
                        );
                        W
                    ];
                    H
                ]))
                .unwrap(),
            _ => println!("Invalid: {}", input.trim()),
        }
    }
}
