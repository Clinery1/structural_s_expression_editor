use s_expression_parser::{
    File as SFile,
};
use crossterm::{
    terminal::{
        EnterAlternateScreen,
        LeaveAlternateScreen,
        enable_raw_mode,
        disable_raw_mode,
    },
    event::{
        read as read_event,
        Event,
        KeyCode,
    },
    cursor::{
        Show as ShowCursor,
        Hide as HideCursor,
        MoveTo as MoveCursorTo,
    },
    execute,
};
use tui::{
    style::Style,
    widgets::Paragraph,
    backend::CrosstermBackend,
    Terminal,
};
use std::{
    fmt::{
        Write,
        Display,
        Formatter,
        Result as FmtResult,
    },
    fs::{
        write as write_file,
        read_to_string,
    },
    mem::swap,
    env::args,
    io::stdout,
};
use object::*;
use colors::*;


mod object;
mod colors;


enum Mode {
    Edit(usize),
    Structural(usize),
    Command(usize),
}
impl Mode {
    pub fn char(&self)->char {
        match self {
            Self::Edit(_)=>'E',
            Self::Structural(_)=>'S',
            Self::Command(_)=>'C',
        }
    }
    pub fn set_command(&mut self) {
        match self {
            Self::Edit(count)|Self::Structural(count)=>*self=Self::Command(*count),
            _=>{},
        }
    }
    pub fn set_structure(&mut self) {
        match self {
            Self::Edit(count)|Self::Command(count)=>*self=Self::Structural(*count),
            _=>{},
        }
    }
    pub fn set_edit(&mut self) {
        match self {
            Self::Structural(count)|Self::Command(count)=>*self=Self::Edit(*count),
            _=>{},
        }
    }
    pub fn len(&self)->usize {
        match self {
            Self::Edit(count)|
                Self::Structural(count)|
                Self::Command(count)=>*count,
        }
    }
    pub fn is_edit(&self)->bool {
        match self {
            Self::Edit(_)=>true,
            _=>false,
        }
    }
    pub fn is_command(&self)->bool {
        match self {
            Self::Command(_)=>true,
            _=>false,
        }
    }
    pub fn is_structural(&self)->bool {
        match self {
            Self::Structural(_)=>true,
            _=>false,
        }
    }
    pub fn sub(&mut self,amt:usize) {
        match self {
            Self::Edit(count)|
                Self::Structural(count)|
                Self::Command(count)=>*count-=amt,
        }
    }
    pub fn add(&mut self,amt:usize) {
        match self {
            Self::Edit(count)|
                Self::Structural(count)|
                Self::Command(count)=>*count+=amt,
        }
    }
    pub fn set(&mut self,amt:usize) {
        match self {
            Self::Edit(count)|
                Self::Structural(count)|
                Self::Command(count)=>*count=amt,
        }
    }
}
impl Display for Mode {
    fn fmt(&self,f:&mut Formatter)->FmtResult {
        match self {
            Self::Edit(_)=>write!(f,"Edit"),
            Self::Structural(_)=>write!(f,"Strctural"),
            Self::Command(_)=>write!(f,"Command"),
        }
    }
}




/*
macro_rules! gen_obj {
    ([number $item:literal])=>{
        Object::Number($item.to_string())
    };
    ([string $item:literal])=>{
        Object::String($item.to_string())
    };
    ([ident $item:literal])=>{
        Object::Ident($item.to_string())
    };
    ($item:ident)=>{
        Object::Ident(stringify!($item).to_string())
    };
    (($($inner:tt)*))=>{
        Object::List(gen_list!($($inner)*))
    };
}
macro_rules! gen_list {
    ($($item:tt)*)=>{
        vec![$(gen_obj!($item),)*]
    };
}
*/


fn main() {
    let mut filename;
    let mut contents=if let Some(name)=args().skip(1).next() {
        filename=Some(name);
        let file_contents=read_to_string(filename.as_ref().unwrap()).unwrap();
        let file=SFile::parse_file(&file_contents).unwrap();
        file.items.into_iter().map(|o|o.into()).collect()
    } else {
        filename=None;
        Vec::new()
    };
    let mut term=Terminal::new(CrosstermBackend::new(stdout())).unwrap();
    let mut cursor=vec![0];
    let colors=Colors::default();
    let mut mode=Mode::Structural(contents.len());
    #[cfg(debug_assertions)]
    let mut debug_log:Vec<String>=Vec::new();
    let mut changed=true;
    let mut saved=true;
    let mut command=String::new();
    let mut command_cursor=1;
    execute!(term.backend_mut(),EnterAlternateScreen).unwrap();
    enable_raw_mode().unwrap();
    'main:loop {
        if changed {
            #[cfg(debug_assertions)]
            debug_log.push(format!("Command cursor: {}; Item length: {}; Mode: {}; Cursor: {}:{:?}",command_cursor,mode.len(),mode,cursor[0],&cursor[1..]));
            let mut y=0;
            term.draw(|f|{
                let mut size=f.size();
                size.height-=2;
                #[cfg(debug_assertions)]
                {size.height/=2}
                if !mode.is_command() {
                    f.render_widget(ObjectWidget::new(&contents,&colors,&cursor),size);
                } else {
                    f.render_widget(ObjectWidget::new(&contents,&colors,&[]),size);
                }
                #[cfg(debug_assertions)]
                {
                    size.y+=size.height;
                    for line in debug_log.iter().rev() {
                        if size.height==0 {
                            break;
                        }
                        f.render_widget(Paragraph::new(line.as_str()),size);
                        size.y+=1;
                        size.height-=1;
                    }
                }
                size.y+=size.height;
                size.height=1;
                let right_size=if saved {
                    format!("| | {}/{}",cursor.last().unwrap(),mode.len())
                } else {
                    format!("| {}/{}",cursor.last().unwrap(),mode.len())
                };
                let left=format!("{} | {}{} |",
                    mode.char(),
                    filename.as_ref().map(String::as_ref).unwrap_or("No name"),
                    if saved {""}else{"*"},
                );
                f.render_widget(
                    Paragraph::new(
                        format!("{}{:>2$}",
                            left,
                            right_size,
                            (size.width-2) as usize-left.chars().count()
                        )
                    ).style(Style::reset().bg(colors.statusline)),
                    size,
                );
                size.y+=1;
                y=size.y;
                f.render_widget(Paragraph::new(command.as_str()),size);
            }).unwrap();
            if mode.is_command() {
                execute!(term.backend_mut(),ShowCursor,MoveCursorTo(command_cursor as u16,y)).unwrap();
            }
            changed=false;
        }
        match read_event().unwrap() {
            Event::Key(event)=>{
                // if !mode.is_command() {
                //     command=format!("Event: {:?}",event);
                // }
                match event.code {
                    // Movement
                    KeyCode::Esc=>if mode.is_command() {
                        command_cursor=1;
                        command=String::new();
                        execute!(term.backend_mut(),HideCursor).unwrap();
                        mode.set_structure();
                        changed=true;
                    } else {
                        mode.set_structure();
                        if cursor.len()>1 {
                            cursor.pop();
                        }
                        #[cfg(debug_assertions)]
                        {mode=make_valid(&mut cursor,&contents,&mut debug_log)}
                        #[cfg(not(debug_assertions))]
                        {mode=make_valid(&mut cursor,&contents)}
                        changed=true;
                    },
                    KeyCode::Right if mode.is_edit()=>{
                        if *cursor.last().unwrap()<mode.len() {
                            *cursor.last_mut().unwrap()+=1;
                            changed=true;
                        }
                    },
                    KeyCode::Left if mode.is_edit()=>{
                        if *cursor.last().unwrap()>0 {
                            *cursor.last_mut().unwrap()-=1;
                            changed=true;
                        }
                    },
                    KeyCode::Enter if mode.is_structural()=>{
                        cursor.push(0);
                        #[cfg(debug_assertions)]
                        {mode=make_valid(&mut cursor,&contents,&mut debug_log)}
                        #[cfg(not(debug_assertions))]
                        {mode=make_valid(&mut cursor,&contents)}
                        changed=true;
                    },
                    // KeyCode::Up=>if !mode.is_command() {
                    //     if cursor.len()>1 {
                    //         cursor.pop();
                    //         #[cfg(debug_assertions)]
                    //         {mode=make_valid(&mut cursor,&contents,&mut debug_log)}
                    //         #[cfg(not(debug_assertions))]
                    //         {mode=make_valid(&mut cursor,&contents)}
                    //         changed=true;
                    //     }
                    // },
                    // KeyCode::Down=>if !mode.is_command() {
                    //     cursor.push(0);
                    //     #[cfg(debug_assertions)]
                    //     {mode=make_valid(&mut cursor,&contents,&mut debug_log)}
                    //     #[cfg(not(debug_assertions))]
                    //     {mode=make_valid(&mut cursor,&contents)}
                    //     changed=true;
                    // },
                    KeyCode::Tab=>if !mode.is_command() {
                        if *cursor.last().unwrap()<mode.len() {
                            *cursor.last_mut().unwrap()+=1;
                            changed=true;
                        }
                    } else {
                        if command_cursor<command.len() {
                            changed=true;
                            command_cursor+=1;
                        }
                    },
                    KeyCode::BackTab=>if !mode.is_command() {
                        if *cursor.last().unwrap()>0 {
                            *cursor.last_mut().unwrap()-=1;
                            changed=true;
                        }
                    } else {
                        if command_cursor>0 {
                            changed=true;
                            command_cursor-=1;
                        }
                    },
                    // Editing an object
                    KeyCode::Char(c) if mode.is_edit()=>{
                        contents[cursor[0]].add_char(&cursor[1..],c);
                        *cursor.last_mut().unwrap()+=1;
                        mode.add(1);
                        changed=true;
                        saved=false;
                    },
                    KeyCode::Backspace if mode.is_edit()=>{
                        if *cursor.last().unwrap()>0&&mode.len()>0 {
                            *cursor.last_mut().unwrap()-=1;
                            contents[cursor[0]].remove(&cursor[1..]);
                            mode.sub(1);
                            changed=true;
                            saved=false;
                        }
                    },
                    KeyCode::Delete=>if mode.is_edit() {
                        if mode.len()>0 {
                            contents[cursor[0]].remove(&cursor[1..]);
                            mode.sub(1);
                            changed=true;
                            saved=false;
                        }
                    } else if mode.is_structural() {
                        if mode.len()>0 {
                            if cursor.len()==1 {
                                if cursor[0]<contents.len() {
                                    contents.remove(cursor[0]);
                                    mode.sub(1);
                                    changed=true;
                                    saved=false;
                                }
                            } else {
                                if *cursor.last().unwrap()<mode.len() {
                                    contents[cursor[0]].remove(&cursor[1..]);
                                    mode.sub(1);
                                    changed=true;
                                    saved=false;
                                }
                            }
                        }
                    } else {
                        if command.len()>command_cursor {
                            command.remove(command_cursor);
                            #[cfg(debug_assertions)]
                            debug_log.push(format!("Command: `{}`",command));
                            changed=true;
                        }
                    },
                    // Command things
                    KeyCode::Char(':') if mode.is_structural()=>{
                        command=":".to_string();
                        mode.set_command();
                        #[cfg(debug_assertions)]
                        debug_log.push(format!("Set mode to command"));
                        execute!(term.backend_mut(),ShowCursor).unwrap();
                        changed=true;
                    },
                    KeyCode::Char(c) if mode.is_command()=>{
                        if command_cursor==command.len() {
                            command.push(c);
                        } else {
                            command.insert(command_cursor,c);
                        }
                        changed=true;
                        #[cfg(debug_assertions)]
                        debug_log.push(format!("Command: `{}`",command));
                        command_cursor+=1;
                    },
                    KeyCode::Backspace if mode.is_command()=>{
                        if command_cursor>0 {
                            if command_cursor==command.len() {
                                command.pop();
                                command_cursor-=1;
                            } else {
                                command_cursor-=1;
                                command.remove(command_cursor);
                            }
                            #[cfg(debug_assertions)]
                            debug_log.push(format!("Command: `{}`",command));
                            changed=true;
                        }
                    },
                    KeyCode::Enter if mode.is_command()=>{
                        let mut c=String::new();
                        swap(&mut c,&mut command);
                        let args=c[1..].split(' ').collect::<Vec<_>>();
                        match args[0] {
                            "q"|"wq"|"q!"|"wq!"|"w"=>{
                                let force=args[0].contains('!');
                                let quit=args[0].contains('q');
                                let write=args[0].contains('w');
                                if write {
                                    if args.len()>1 {
                                        filename=Some(args[1].to_string());
                                    }
                                    if let Some(filename)=&filename {
                                        let mut out=String::new();
                                        for obj in contents.iter() {
                                            writeln!(out,"{}",obj).unwrap();
                                        }
                                        let lines=out.lines().count();
                                        let bytes=out.len();
                                        if let Err(e)=write_file(&filename,out) {
                                            command=format!("Could not save file. Reason: {}",e);
                                        } else {
                                            command=format!("`{}` {} lines, {} bytes",filename,lines,bytes);
                                            saved=true;
                                        }
                                    } else {
                                        command=format!("No file name");
                                    }
                                }
                                if quit {
                                    if saved||force {
                                        break 'main;
                                    } else if command.len()==0 {
                                        if let Some(filename)=&filename {
                                            command=format!("File `{}` was not saved. To force quit, do `:q!`",filename);
                                        } else {
                                            command=format!("Buffer was not saved to a file. To force quit, do `:q!`");
                                        }
                                    }
                                }
                            },
                            _=>{},
                        }
                        command_cursor=1;
                        execute!(term.backend_mut(),HideCursor).unwrap();
                        mode.set_structure();
                        changed=true;
                    },
                    // Deleting an object
                    // Adding an object
                    KeyCode::Char('l') if mode.is_structural()=>{
                        let obj=Object::List(Vec::new());
                        if mode.len()>0&&*cursor.last().unwrap()<mode.len() {*cursor.last_mut().unwrap()+=1}
                        if cursor.len()==1 {
                            if cursor[0]+1>contents.len() {
                                contents.push(obj);
                            } else {
                                contents.insert(cursor[0],obj);
                            }
                        } else {
                            contents[cursor[0]].add_object(&cursor[1..],obj);
                        }
                        mode.set(0);
                        cursor.push(0);
                        changed=true;
                        saved=false;
                    },
                    KeyCode::Char('"') if mode.is_structural()=>{
                        let obj=Object::String(String::new());
                        if mode.len()>0&&*cursor.last().unwrap()<mode.len() {*cursor.last_mut().unwrap()+=1}
                        if cursor.len()==1 {
                            if cursor[0]>contents.len() {
                                contents.push(obj);
                            } else {
                                contents.insert(cursor[0],obj);
                            }
                        } else {
                            contents[cursor[0]].add_object(&cursor[1..],obj);
                        }
                        mode.set(0);
                        mode.set_edit();
                        cursor.push(0);
                        changed=true;
                        saved=false;
                    },
                    KeyCode::Char('i') if mode.is_structural()=>{
                        let obj=Object::Ident(String::new());
                        if mode.len()>0&&*cursor.last().unwrap()<mode.len() {*cursor.last_mut().unwrap()+=1}
                        if cursor.len()==1 {
                            if cursor[0]>contents.len() {
                                contents.push(obj);
                            } else {
                                contents.insert(cursor[0],obj);
                            }
                        } else {
                            contents[cursor[0]].add_object(&cursor[1..],obj);
                        }
                        mode.set(0);
                        mode.set_edit();
                        cursor.push(0);
                        changed=true;
                        saved=false;
                    },
                    KeyCode::Char('n') if mode.is_structural()=>{
                        let obj=Object::Number("0".into());
                        if mode.len()>0&&*cursor.last().unwrap()<mode.len() {*cursor.last_mut().unwrap()+=1}
                        if cursor.len()==1 {
                            if cursor[0]>contents.len() {
                                contents.push(obj);
                            } else {
                                contents.insert(cursor[0],obj);
                            }
                        } else {
                            contents[cursor[0]].add_object(&cursor[1..],obj);
                        }
                        mode.set(0);
                        mode.set_edit();
                        cursor.push(0);
                        changed=true;
                        saved=false;
                    },
                    _=>{},
                }
            },
            _=>{},
        }
    }
    disable_raw_mode().unwrap();
    execute!(term.backend_mut(),LeaveAlternateScreen).unwrap();
}
/// Returns Ok(count) for edit mode and Err(count) for just valid
#[cfg(debug_assertions)]
fn make_valid(cursor:&mut Vec<usize>,objs:&[Object],debug_log:&mut Vec<String>)->Mode {
    use CursorValidReason::*;
    if objs.len()==0 {
        debug_log.push(format!("There are no objects, so set cursor to zero"));
        cursor.truncate(1);
        cursor[0]=0;
        return Mode::Structural(0);
    }
    if cursor[0]>=objs.len() {
        debug_log.push(format!("Cursor was past the end of the object list"));
        cursor.truncate(1);
        cursor[0]=objs.len()-1;
    }
    if cursor.len()==1 {
        return Mode::Structural(objs.len());
    }
    for _ in 0..4 { // four tries to get the cursor valid
        match objs[cursor[0]].is_cursor_valid(&cursor[1..]) {
            Valid(count)=>return Mode::Structural(count),
            Edit(count)=>return Mode::Edit(count),
            OutOfRange(count)=>{
                debug_log.push(format!("Out of range: {}",count));
                *cursor.last_mut().unwrap()-=count;
            },
            DoesNotExist(count)=>{
                debug_log.push(format!("Does not exist: {}",count));
                cursor.truncate(cursor.len()-count);
            },
        }
    }
    panic!("Cannot make the cursor valid");
}
#[cfg(not(debug_assertions))]
fn make_valid(cursor:&mut Vec<usize>,objs:&[Object])->Mode {
    use CursorValidReason::*;
    if objs.len()==0 {
        cursor.truncate(1);
        cursor[0]=0;
        return Mode::Structural(0);
    }
    if cursor[0]>=objs.len() {
        cursor.truncate(1);
        cursor[0]=objs.len()-1;
    }
    if cursor.len()==1 {
        return Mode::Structural(objs.len());
    }
    for _ in 0..4 { // four tries to get the cursor valid
        match objs[cursor[0]].is_cursor_valid(&cursor[1..]) {
            Valid(count)=>return Mode::Structural(count),
            Edit(count)=>return Mode::Edit(count),
            OutOfRange(count)=>{
                *cursor.last_mut().unwrap()-=count;
            },
            DoesNotExist(count)=>{
                cursor.truncate(cursor.len()-count);
            },
        }
    }
    panic!("Cannot make the cursor valid");
}
