use tui::{
    style::{
        Style,
        Color,
    },
    layout::Rect,
    buffer::Buffer,
    widgets::Widget,
};
use super::*;


pub struct ObjectWidget<'obj> {
    objects:&'obj [Object],
    colors:&'obj Colors,
    cursor:&'obj [usize],
}
impl<'obj> ObjectWidget<'obj> {
    pub fn new(objects:&'obj [Object],colors:&'obj Colors,cursor:&'obj [usize])->Self {
        Self{objects,colors,cursor}
    }
}
impl<'obj> Widget for ObjectWidget<'obj> {
    fn render(self,area:Rect,buf:&mut Buffer) {
        let mut line=0;
        buf.reset();
        if self.cursor.len()>0 {
            for (i,object) in self.objects.iter().enumerate() {
                object.render(self.colors,&mut line,0,0,buf,if i==self.cursor[0]{Some(&self.cursor[1..])}else{None});
                line+=1;
                if line>area.height {break}
            }
            if self.cursor[0]==self.objects.len() {
                let blank_style=Style::reset()
                    .fg(Color::Rgb(0,0,0))
                    .bg(self.colors.ident);
                buf.set_stringn(
                    0,
                    line,
                    " ",
                    1,
                    blank_style,
                );
            }
        } else {
            for object in self.objects {
                object.render(self.colors,&mut line,0,0,buf,None);
                line+=1;
                if line>area.height {break}
            }
        }
    }
}
