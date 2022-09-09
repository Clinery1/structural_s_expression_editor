use crossterm::{
    terminal::{
        EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    event::read as read_event,
    execute,
};
use tui::{
    style::{
        Style,
        Color,
        Modifier,
    },
    widgets::Widget,
    buffer::Buffer,
    layout::Rect,
    backend::CrosstermBackend,
    Terminal,
};
use std::{
    fmt::{
        Display,
        Formatter,
        Result as FmtResult,
    },
    io::stdout,
    ops::Index,
};


#[derive(Debug)]
enum DisplayObject {
    List(Vec<Self>),
    Ident(String),
    Number(String),
    String(String),
}
impl Display for DisplayObject {
    fn fmt(&self,f:&mut Formatter)->FmtResult {
        let indent=f.width().unwrap_or(0);
        for _ in 0..indent {write!(f," ")?}
        match self {
            Self::List(items)=>{
                match items.as_slice() {
                    [first,last]=>{
                        match last {
                            Self::List(_)=>{
                                writeln!(f,"({}",first)?;
                                write!(f,"{:1$})",last,indent+4)
                            },
                            _=>write!(f,"({} {})",first,last),
                        }
                    },
                    [first,rest@..]=>{
                        write!(f,"({}",first)?;
                        for i in rest {
                            writeln!(f)?;
                            write!(f,"{:1$}",i,indent+4)?;
                        }
                        write!(f,")")
                    },
                    []=>{
                        write!(f,"()")
                    },
                }
            },
            Self::Ident(s)|Self::Number(s)=>f.write_str(s),
            Self::String(s)=>write!(f,"\"{}\"",s),
        }
    }
}
impl DisplayObject {
    pub fn render(&self,colors:&Colors,line:&mut u16,level:u16,offset:u16,buf:&mut Buffer)->u16 {
        match self {
            Self::Ident(s)=>{
                let indent=(level*4)+offset;
                buf.set_stringn(
                    indent,   // make the indent 4 spaces
                    *line,
                    s,
                    s.len(),
                    Style::reset()
                        .fg(colors.ident)
                        .add_modifier(Modifier::ITALIC),
                ).0
            },
            Self::String(s)=>{
                let indent=(level*4)+offset;
                let style=Style::reset().fg(colors.string);
                let mut last_column=buf.set_stringn(indent,*line,"\"",1,style).0;
                last_column=buf.set_stringn(
                    last_column,   // make the indent 4 spaces
                    *line,
                    s,
                    s.len(),
                    style,
                ).0;
                buf.set_stringn(last_column,*line,"\"",1,style).0
            },
            Self::Number(s)=>{
                let indent=(level*4)+offset;
                buf.set_stringn(
                    indent,   // make the indent 4 spaces
                    *line,
                    s,
                    s.len(),
                    Style::reset().fg(colors.number),
                ).0
            },
            Self::List(items)=>{
                let style=Style::reset().fg(colors[level]);
                match items.as_slice() {
                    [first,last]=>{
                        match last {
                            Self::List(_)=>{    // print the list on a multiple lines
                                let indent=(level*4)+offset;
                                buf.set_stringn(
                                    indent,
                                    *line,
                                    "(",
                                    1,
                                    style,
                                );
                                first.render(colors,line,level,1,buf);
                                *line+=1;
                                let last_column=last.render(colors,line,level+1,0,buf);
                                buf.set_stringn(
                                    last_column,
                                    *line,
                                    ")",
                                    1,
                                    style,
                                ).0
                            },
                            _=>{    // print the list on one line
                                let indent=(level*4)+offset;
                                buf.set_stringn(
                                    indent,
                                    *line,
                                    "(",
                                    1,
                                    style,
                                );
                                let mut last_column=first.render(colors,line,level,1,buf)+1;
                                last_column=last.render(colors,line,level,last_column-indent,buf);
                                buf.set_stringn(
                                    last_column,
                                    *line,
                                    ")",
                                    1,
                                    style,
                                ).0
                            },
                        }
                    },
                    [first,rest@..]=>{
                        let indent=(level*4)+offset;
                        buf.set_stringn(
                            indent,
                            *line,
                            "(",
                            1,
                            style,
                        );
                        let mut last_column=first.render(colors,line,level,1,buf);
                        for item in rest {
                            *line+=1;
                            last_column=item.render(colors,line,level+1,0,buf);
                        }
                        buf.set_stringn(
                            last_column,
                            *line,
                            ")",
                            1,
                            style,
                        ).0
                    },
                    []=>{
                        let indent=(level*4)+offset;
                        buf.set_stringn(
                            indent,
                            *line,
                            "()",
                            2,
                            style,
                        ).0
                    },
                }
            },
        }
    }
}


struct DisplayObjectWidget<'obj> {
    objects:&'obj [DisplayObject],
    colors:&'obj Colors,
}
impl<'obj> Widget for DisplayObjectWidget<'obj> {
    fn render(self,area:Rect,buf:&mut Buffer) {
        let mut line=0;
        buf.reset();
        for object in self.objects {
            object.render(self.colors,&mut line,0,0,buf);
            line+=1;
            if line>area.width {break}
        }
    }
}
#[derive(Debug,Clone)]
struct Colors {
    rainbow:Vec<Color>,
    pub ident:Color,
    pub number:Color,
    pub string:Color,
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
        }
    }
}
impl<T:Into<usize>> Index<T> for Colors {
    type Output=Color;
    fn index(&self,i:T)->&Self::Output {
        &self.rainbow[i.into()%self.rainbow.len()]
    }
}


macro_rules! gen_obj {
    ([number $item:literal])=>{
        DisplayObject::Number($item.to_string())
    };
    ([string $item:literal])=>{
        DisplayObject::String($item.to_string())
    };
    ([ident $item:literal])=>{
        DisplayObject::Ident($item.to_string())
    };
    ($item:ident)=>{
        DisplayObject::Ident(stringify!($item).to_string())
    };
    (($($inner:tt)*))=>{
        DisplayObject::List(gen_list!($($inner)*))
    };
}
macro_rules! gen_list {
    ($($item:tt)*)=>{
        vec![$(gen_obj!($item),)*]
    };
}


fn main() {
    let mut term=Terminal::new(CrosstermBackend::new(stdout())).unwrap();
    execute!(term.backend_mut(),EnterAlternateScreen);
    term.draw(|f|{
        use DisplayObject as Obj;
        let test_data:Vec<Obj>=gen_list!(
            ([ident "!!DOCTYPE"]
                html)
            (html
                (head
                    (title [string "My website"]))
                (body
                    (h1 [string "This is a title"])
                    ([ident "!img"] (src [string "./bunny.jpg"]) (id [string "bunny"]))))
        );
        let colors=Colors::default();
        f.render_widget(DisplayObjectWidget{objects:&test_data,colors:&colors},f.size());
    }).unwrap();
    read_event().unwrap();
    execute!(term.backend_mut(),LeaveAlternateScreen);
}
