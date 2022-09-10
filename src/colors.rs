use std::ops::Index;
use tui::style::Color;


#[derive(Debug,Clone)]
pub struct Colors {
    rainbow:Vec<Color>,
    pub ident:Color,
    pub number:Color,
    pub string:Color,
    pub statusline:Color,
}
impl Default for Colors {
    fn default()->Self {
        let red=Color::Rgb(0xFB,0x46,0x7B);
        let cyan=Color::Rgb(0x80,0xA0,0xFF);
        let purple=Color::Rgb(0x97,0x5E,0xEC);
        let yellow=Color::Rgb(0xFF,0xCC,0x00);
        let aqua=Color::Rgb(0x00,0xD5,0xA7);
        let orange=Color::Rgb(0xFF,0x8D,0x03);
        let green=Color::Rgb(0xB8,0xEE,0x92);
        let white=Color::Rgb(0xCE,0xD5,0xE5);
        let grey=Color::Rgb(0x49,0x46,0x46);
        Colors {
            rainbow:vec![
                red,
                cyan,
                purple,
                yellow,
                aqua,
                orange,
                green,
            ],
            ident:white,
            number:red,
            string:green,
            statusline:grey,
        }
    }
}
impl<T:Into<usize>> Index<T> for Colors {
    type Output=Color;
    fn index(&self,i:T)->&Self::Output {
        &self.rainbow[i.into()%self.rainbow.len()]
    }
}
