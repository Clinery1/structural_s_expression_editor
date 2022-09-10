use s_expression_parser::Object as SObject;
use tui::{
    style::{
        Style,
        Color,
    },
    buffer::Buffer,
};
use std::{
    fmt::{
        Display,
        Formatter,
        Result as FmtResult,
    },
    mem::swap,
};
use crate::colors::*;
pub use widget::*;


mod widget;


pub enum CursorValidReason {
    /// The length of the object. Lists are the amount of objects, String, Number, and Ident are
    /// all the amount of chars in the string.
    Valid(usize),
    /// The amount of chars in the object.
    Edit(usize),
    /// The search was valid, but the cursor was out of range of the container by n units.
    OutOfRange(usize),
    /// The search was NOT valid and the last n positions do not exist.
    DoesNotExist(usize),
}
#[derive(Debug)]
pub enum Object {
    List(Vec<Self>),
    Ident(String),
    Number(String),
    String(String),
}
impl<'input> From<SObject<'input>> for Object {
    fn from(o:SObject<'input>)->Self {
        match o {
           SObject::Ident(_,i,_)=>Self::Ident(i.to_string()),
           SObject::String(_,mut s,_)=>{
               s=s.replace('\\',"\\\\");
               s=s.replace('\n',"\\n");
               s=s.replace('\r',"\\r");
               s=s.replace('\t',"\\t");
               s=s.replace('"',"\\\"");
               s=s.replace('\0',"\\0");
               Self::String(s)
           },
           SObject::Number(_,n,_)=>Self::Number(n.to_string()),
           SObject::List(_,items,_)=>Self::List(items.into_iter().map(|i|i.into()).collect()),
        }
    }
}
impl Display for Object {
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
impl Object {
    pub fn is_cursor_valid(&self,cursor:&[usize])->CursorValidReason {
        use CursorValidReason::*;
        if cursor.len()==1 {
            match self {
                Self::List(items)=>if items.len()>=cursor[0]||(items.len()==0&&cursor[0]==0) {
                    Valid(items.len())
                } else {
                    OutOfRange(cursor[0]-items.len())
                },
                Self::Ident(s)|Self::String(s)|Self::Number(s)=>{
                    let count=s.chars().count();
                    if count>=cursor[0]||(count==0&&cursor[0]==0) {
                        Edit(count)
                    } else {
                        OutOfRange(cursor[0]-count)
                    }
                },
            }
        } else if cursor.len()==0 {
            match self {
                Self::List(items)=>Valid(items.len()),
                Self::Ident(s)|Self::String(s)|Self::Number(s)=>Valid(s.chars().count()),
            }
        } else {
            match self {
                Self::List(items)=>{
                    if items.len()<=cursor[0] {
                        return DoesNotExist(cursor.len()-1);
                    }
                    return items[cursor[0]].is_cursor_valid(&cursor[1..]);
                },
                _=>{
                    DoesNotExist(cursor.len()-1)
                },
            }
        }
    }
    pub fn add_object(&mut self,cursor:&[usize],obj:Self) {
        if cursor.len()<=1 {
            match self {
                Self::List(items)=>{
                    if cursor.len()==0 {
                        items.push(obj);
                    } else {
                        if cursor[0]>=items.len() {
                            items.push(obj);
                        } else {
                            items.insert(cursor[0],obj);
                        }
                    }
                },
                item=>{
                    let mut old_item=Self::List(Vec::new());
                    swap(item,&mut old_item);
                    match item {
                        Self::List(items)=>{
                            items.push(old_item);
                            items.push(obj);
                        },
                        _=>unreachable!(),
                    }
                },
            }
        } else {
            match self {
                Self::List(items)=>items[cursor[0]].add_object(&cursor[1..],obj),
                _=>todo!("invalid cursor position"),
            }
        }
    }
    pub fn add_char(&mut self,cursor:&[usize],c:char) {
        if cursor.len()<=1 {
            match self {
                Self::List(_)=>todo!("Attempt to add a char to a list of objects"),
                Self::Ident(s)|Self::String(s)|Self::Number(s)=>{
                    if cursor.len()==0 {
                        s.push(c);
                    } else {
                        if cursor[0]+1>=s.chars().count() {
                            s.push(c);
                        } else {
                            let index=s.char_indices().nth(cursor[0]).expect("Invalid cursor position in non-list object").0;
                            s.insert(index,c);
                        }
                    }
                },
            }
        } else {
            match self {
                Self::List(items)=>items[cursor[0]].add_char(&cursor[1..],c),
                _=>todo!("invalid cursor position"),
            }
        }
    }
    pub fn remove(&mut self,cursor:&[usize]) {
        if cursor.len()==1 {
            match self {
                Self::List(items)=>{
                    if items.len()>0 {
                        items.remove(cursor[0]);
                    }
                },
                Self::Ident(s)|Self::String(s)|Self::Number(s)=>{
                    if s.len()>0 {  // dont allow the last char to be removed
                        if cursor[0]==0 {
                            s.remove(0);
                        } else {
                            if let Some(index)=s.char_indices().nth(cursor[0]) {
                                s.remove(index.0);
                            }
                        }
                    }
                },
            }
        } else if cursor.len()==0 {
            // do nothing, because there is nothing to do
        } else {
            match self {
                Self::List(items)=>items[cursor[0]].remove(&cursor[1..]),
                _=>todo!("invalid cursor position"),
            }
        }
    }
    pub fn render(&self,colors:&Colors,line:&mut u16,level:u16,offset:u16,buf:&mut Buffer,cursor:Option<&[usize]>)->u16 {
        match self {
            Self::Ident(s)=>{
                let indent=(level*4)+offset;
                let s=if s.len()==0 {
                    "(I)"
                } else {
                    s.as_str()
                };
                if let Some(cursor)=cursor {
                    if cursor.len()==1 {
                        if cursor[0]==s.chars().count() {
                            let last=buf.set_stringn(
                                indent, // make the indent 4 spaces
                                *line,
                                s,
                                s.len(),
                                Style::reset()
                                    .fg(colors.ident)
                            ).0;
                            buf.get_mut(last,*line).bg=colors.ident;
                            buf.get_mut(last,*line).fg=Color::Rgb(0,0,0);
                            return last;
                        } else {
                            let mut cindices=s.char_indices().skip(cursor[0]);
                            let index=cindices.next().expect("Invalid cursor position").0;
                            let after=cindices.next().unwrap_or((s.len(),' ')).0;
                            let cursor_style=Style::reset()
                                .fg(Color::Rgb(0,0,0))
                                .bg(colors.ident);
                            let other_style=Style::reset()
                                .fg(colors.ident);
                            let mut last_char=buf.set_stringn(
                                indent, // make the indent 4 spaces
                                *line,
                                &s[..index],
                                s.len(),
                                other_style,
                            ).0;
                            last_char=buf.set_stringn(
                                last_char,  // make the indent 4 spaces
                                *line,
                                &s[index..after],
                                s.len(),
                                cursor_style,
                            ).0;
                            buf.set_stringn(
                                last_char,  // make the indent 4 spaces
                                *line,
                                &s[after..],
                                s.len(),
                                other_style,
                            ).0
                        }
                    } else {
                        buf.set_stringn(
                            indent, // make the indent 4 spaces
                            *line,
                            s,
                            s.len(),
                            Style::reset()
                                .fg(Color::Rgb(0,0,0))
                                .bg(colors.ident)
                        ).0
                    }
                } else {
                    buf.set_stringn(
                        indent, // make the indent 4 spaces
                        *line,
                        s,
                        s.len(),
                        Style::reset().fg(colors.ident),
                    ).0
                }
            },
            Self::String(s)=>{
                let indent=(level*4)+offset;
                if let Some(cursor)=cursor {
                    if cursor.len()==1 {
                        if cursor==&[s.len()] {
                            let style=Style::reset()
                                .fg(colors.string);
                            let mut last_column=buf.set_stringn(indent,*line,"\"",1,style).0;
                            last_column=buf.set_stringn(
                                last_column,    // make the indent 4 spaces
                                *line,
                                s,
                                s.len(),
                                style,
                            ).0;
                            buf.set_stringn(last_column,*line,"\"",1,style.bg(colors.string).fg(Color::Rgb(0,0,0))).0
                        } else {
                            let mut cindices=s.char_indices().skip(cursor[0]);
                            let index=cindices.next().expect("Invalid cursor position").0;
                            let after=cindices.next().unwrap_or((s.len(),' ')).0;
                            let cursor_style=Style::reset()
                                .fg(Color::Rgb(0,0,0))
                                .bg(colors.string);
                            let other_style=Style::reset()
                                .fg(colors.string);
                            let mut last_char=buf.set_stringn(indent,*line,"\"",1,other_style).0;
                            last_char=buf.set_stringn(
                                last_char,  // make the indent 4 spaces
                                *line,
                                &s[..index],
                                s.len(),
                                other_style,
                            ).0;
                            last_char=buf.set_stringn(
                                last_char,  // make the indent 4 spaces
                                *line,
                                &s[index..after],
                                s.len(),
                                cursor_style,
                            ).0;
                            last_char=buf.set_stringn(
                                last_char,  // make the indent 4 spaces
                                *line,
                                &s[after..],
                                s.len(),
                                other_style,
                            ).0;
                            buf.set_stringn(last_char,*line,"\"",1,other_style).0
                        }
                    } else {
                        let style=Style::reset()
                            .fg(Color::Rgb(0,0,0))
                            .bg(colors.string);
                        let mut last_column=buf.set_stringn(indent,*line,"\"",1,style).0;
                        last_column=buf.set_stringn(
                            last_column,    // make the indent 4 spaces
                            *line,
                            s,
                            s.len(),
                            style,
                        ).0;
                        buf.set_stringn(last_column,*line,"\"",1,style).0
                    }
                } else {
                    let style=Style::reset().fg(colors.string);
                    let mut last_column=buf.set_stringn(indent,*line,"\"",1,style).0;
                    last_column=buf.set_stringn(
                        last_column,    // make the indent 4 spaces
                        *line,
                        s,
                        s.len(),
                        style,
                    ).0;
                    buf.set_stringn(last_column,*line,"\"",1,style).0
                }
            },
            Self::Number(s)=>{
                let indent=(level*4)+offset;
                let s=if s.len()==0 {
                    "(N)"
                } else {
                    s.as_str()
                };
                if let Some(cursor)=cursor {
                    if cursor.len()==1 {
                        if cursor[0]==s.chars().count() {
                            let last=buf.set_stringn(
                                indent, // make the indent 4 spaces
                                *line,
                                s,
                                s.len(),
                                Style::reset().fg(colors.number)
                            ).0;
                            buf.get_mut(last,*line).bg=colors.number;
                            buf.get_mut(last,*line).fg=Color::Rgb(0,0,0);
                            return last;
                        } else {
                            let mut cindices=s.char_indices().skip(cursor[0]);
                            let index=cindices.next().expect("Invalid cursor position").0;
                            let after=cindices.next().unwrap_or((s.len(),' ')).0;
                            let cursor_style=Style::reset()
                                .fg(Color::Rgb(0,0,0))
                                .bg(colors.number);
                            let other_style=Style::reset()
                                .fg(colors.number);
                            let mut last_char=buf.set_stringn(
                                indent, // make the indent 4 spaces
                                *line,
                                &s[..index],
                                s.len(),
                                other_style,
                            ).0;
                            last_char=buf.set_stringn(
                                last_char,  // make the indent 4 spaces
                                *line,
                                &s[index..after],
                                s.len(),
                                cursor_style,
                            ).0;
                            buf.set_stringn(
                                last_char,  // make the indent 4 spaces
                                *line,
                                &s[after..],
                                s.len(),
                                other_style,
                            ).0
                        }
                    } else {
                        buf.set_stringn(
                            indent,   // make the indent 4 spaces
                            *line,
                            s,
                            s.len(),
                            Style::reset()
                                .fg(Color::Rgb(0,0,0))
                                .bg(colors.number),
                        ).0
                    }
                } else {
                    buf.set_stringn(
                        indent,   // make the indent 4 spaces
                        *line,
                        s,
                        s.len(),
                        Style::reset().fg(colors.number),
                    ).0
                }
            },
            Self::List(items)=>{
                let style=Style::default()
                    .fg(colors[level]);
                let style_rev=Style::reset()
                    .fg(Color::Rgb(0,0,0))
                    .bg(colors[level]);
                let blank_style=Style::reset()
                    .fg(Color::Rgb(0,0,0))
                    .bg(colors.ident);
                let blank_style_rev=Style::reset();
                match items.as_slice() {
                    [first,last]=>{
                        match last {
                            Self::List(_)=>{    // print the list on a multiple lines
                                if let Some(cursor)=cursor {
                                    if cursor.len()>0 {
                                        let indent=(level*4)+offset;
                                        buf.set_stringn(
                                            indent,
                                            *line,
                                            "(",
                                            1,
                                            style,
                                        );
                                        first.render(colors,line,level,1,buf,if cursor[0]==0 {Some(&cursor[1..])}else{None});
                                        *line+=1;
                                        let mut last_column=last.render(colors,line,level+1,0,buf,if cursor[0]==1 {Some(&cursor[1..])}else{None});
                                        if cursor==&[items.len()] {
                                            last_column=buf.set_stringn(
                                                last_column,
                                                *line,
                                                " ",
                                                1,
                                                blank_style_rev,
                                            ).0;
                                            last_column=buf.set_stringn(
                                                last_column,
                                                *line,
                                                " ",
                                                1,
                                                blank_style,
                                            ).0;
                                        }
                                        buf.set_stringn(
                                            last_column,
                                            *line,
                                            ")",
                                            1,
                                            style,
                                        ).0
                                    } else {
                                        let indent=(level*4)+offset;
                                        buf.set_stringn(
                                            indent,
                                            *line,
                                            "(",
                                            1,
                                            style_rev,
                                        );
                                        first.render(colors,line,level,1,buf,None);
                                        *line+=1;
                                        let last_column=last.render(colors,line,level+1,0,buf,None);
                                        buf.set_stringn(
                                            last_column,
                                            *line,
                                            ")",
                                            1,
                                            style_rev,
                                        ).0
                                    }
                                } else {
                                    let indent=(level*4)+offset;
                                    buf.set_stringn(
                                        indent,
                                        *line,
                                        "(",
                                        1,
                                        style,
                                    );
                                    first.render(colors,line,level,1,buf,None);
                                    *line+=1;
                                    let last_column=last.render(colors,line,level+1,0,buf,None);
                                    buf.set_stringn(
                                        last_column,
                                        *line,
                                        ")",
                                        1,
                                        style,
                                    ).0
                                }
                            },
                            _=>{    // print the list on one line
                                if let Some(cursor)=cursor {
                                    if cursor.len()>0 {
                                        let indent=(level*4)+offset;
                                        buf.set_stringn(
                                            indent,
                                            *line,
                                            "(",
                                            1,
                                            style,
                                        );
                                        let mut last_column=first.render(colors,line,level,1,buf,if cursor[0]==0 {Some(&cursor[1..])}else{None})+1;
                                        last_column=last.render(colors,line,level,last_column-indent,buf,if cursor[0]==1 {Some(&cursor[1..])}else{None});
                                        if cursor==&[items.len()] {
                                            last_column=buf.set_stringn(
                                                last_column,
                                                *line,
                                                " ",
                                                1,
                                                blank_style_rev,
                                            ).0;
                                            last_column=buf.set_stringn(
                                                last_column,
                                                *line,
                                                " ",
                                                1,
                                                blank_style,
                                            ).0;
                                        }
                                        buf.set_stringn(
                                            last_column,
                                            *line,
                                            ")",
                                            1,
                                            style,
                                        ).0
                                    } else {
                                        let indent=(level*4)+offset;
                                        buf.set_stringn(
                                            indent,
                                            *line,
                                            "(",
                                            1,
                                            style_rev,
                                        );
                                        let mut last_column=first.render(colors,line,level,1,buf,None)+1;
                                        last_column=last.render(colors,line,level,last_column-indent,buf,None);
                                        buf.set_stringn(
                                            last_column,
                                            *line,
                                            ")",
                                            1,
                                            style_rev,
                                        ).0
                                    }
                                } else {
                                    let indent=(level*4)+offset;
                                    buf.set_stringn(
                                        indent,
                                        *line,
                                        "(",
                                        1,
                                        style,
                                    );
                                    let mut last_column=first.render(colors,line,level,1,buf,None)+1;
                                    last_column=last.render(colors,line,level,last_column-indent,buf,None);
                                    buf.set_stringn(
                                        last_column,
                                        *line,
                                        ")",
                                        1,
                                        style,
                                    ).0
                                }
                            },
                        }
                    },
                    [first,rest@..]=>{
                        if let Some(cursor)=cursor {
                            if cursor.len()>0 {
                                let indent=(level*4)+offset;
                                buf.set_stringn(
                                    indent,
                                    *line,
                                    "(",
                                    1,
                                    style,
                                );
                                let mut last_column=first.render(colors,line,level,1,buf,if cursor[0]==0 {Some(&cursor[1..])}else{None});
                                for (i,item) in rest.iter().enumerate() {
                                    *line+=1;
                                    last_column=item.render(colors,line,level+1,0,buf,if cursor[0]==(i+1) {Some(&cursor[1..])}else{None});
                                }
                                if cursor==&[items.len()] {
                                    last_column=buf.set_stringn(
                                        last_column,
                                        *line,
                                        " ",
                                        1,
                                        blank_style_rev,
                                    ).0;
                                    last_column=buf.set_stringn(
                                        last_column,
                                        *line,
                                        " ",
                                        1,
                                        blank_style,
                                    ).0;
                                }
                                buf.set_stringn(
                                    last_column,
                                    *line,
                                    ")",
                                    1,
                                    style,
                                ).0
                            } else {
                                let indent=(level*4)+offset;
                                buf.set_stringn(
                                    indent,
                                    *line,
                                    "(",
                                    1,
                                    style_rev,
                                );
                                let mut last_column=first.render(colors,line,level,1,buf,None);
                                for item in rest {
                                    *line+=1;
                                    last_column=item.render(colors,line,level+1,0,buf,None);
                                }
                                buf.set_stringn(
                                    last_column,
                                    *line,
                                    ")",
                                    1,
                                    style_rev,
                                ).0
                            }
                        } else {
                            let indent=(level*4)+offset;
                            buf.set_stringn(
                                indent,
                                *line,
                                "(",
                                1,
                                style,
                            );
                            let mut last_column=first.render(colors,line,level,1,buf,None);
                            for item in rest {
                                *line+=1;
                                last_column=item.render(colors,line,level+1,0,buf,None);
                            }
                            buf.set_stringn(
                                last_column,
                                *line,
                                ")",
                                1,
                                style,
                            ).0
                        }
                    },
                    []=>{
                        let indent=(level*4)+offset;
                        if let Some(cursor)=cursor {
                            if cursor==&[items.len()] {
                                let mut last_column=buf.set_stringn(
                                    indent,
                                    *line,
                                    "(",
                                    1,
                                    style,
                                ).0;
                                last_column=buf.set_stringn(
                                    last_column,
                                    *line,
                                    " ",
                                    1,
                                    blank_style,
                                ).0;
                                return buf.set_stringn(
                                    last_column,
                                    *line,
                                    ")",
                                    1,
                                    style,
                                ).0;
                            }
                            if cursor.len()==0||cursor.len()==1 {
                                return buf.set_stringn(
                                    indent,
                                    *line,
                                    "()",
                                    2,
                                    style_rev,
                                ).0;
                            }
                        }
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
