use std::fmt::Display;
use std::io::Error;
use crossterm::event::{Event, KeyCode, MouseEvent, MouseEventKind};
use either::Either;
use immt_system::controller::Controller;
use crate::components::UITab;
use ratatui::{prelude::*, style::palette::tailwind, widgets::*};
use ratatui::layout::Size;
use tui_scrollview::{ScrollView, ScrollViewState};
use immt_api::archives::{ArchiveGroupT, ArchiveId};
use immt_api::archives::ArchiveT;
use immt_api::FinalStr;
use immt_system::backend::archives::{Archive, ArchiveGroup};
use crate::utils::Depth;

pub struct Library {
    scroll_state:ScrollViewState,
    tree_state:TreeState
}
impl Default for Library {
    fn default() -> Self {
        Self {
            scroll_state:ScrollViewState::default(),
            tree_state:TreeState {
                tree:vec!(),
                active:None
            }
        }
    }
}


impl Library {
    fn treeview(&mut self, buf: &mut Buffer,height:u16) {
        //let mut depth = 0;
        let range = (self.scroll_state.offset().y,self.scroll_state.offset().y + height - 1);
        let mut curr = buf.area;
        let mut i = 0;
        while i < self.tree_state.tree.len() {
            let c = &self.tree_state.tree[i];
            let has_next = if let Some(e) = self.tree_state.tree.get(i+1) {
                e.depth >= c.depth
            } else {false};
            if self.tree_state.active == Some(i) {
                Line::default().spans(vec!(Span::styled(format!("⮕ {}{}",Depth(c.depth,has_next),c.render),Style::new().bg(tailwind::LIME.c600).fg(tailwind::BLACK).add_modifier(Modifier::BOLD))))
                    .render(curr,buf);
                if curr.y > range.1 {
                    let mut copy = curr;
                    if height > curr.y + 1 {
                        copy.y = 0;
                    } else {
                        copy.y = curr.y + 1 - height;
                    }
                    self.scroll_state.set_offset(copy.as_position())
                } else if curr.y < range.0 {
                    self.scroll_state.set_offset(curr.as_position())
                }
            } else {
                Line::raw(format!("  {}{}",Depth(c.depth,has_next),c.render)).render(curr,buf);
            }
            curr.y += 1;
            i += 1;
        }
    }
    fn key_down(&mut self) {
        if let Some(i) = self.tree_state.active {
            if i < self.tree_state.tree.len() - 1 {
                self.tree_state.active = Some(i + 1);
            }
        } else {
            self.tree_state.active = Some(0);
        }
    }
    fn key_up(&mut self) {
        if let Some(i) = self.tree_state.active {
            if i > 0 {
                self.tree_state.active = Some(i - 1);
            }
        } else {
            self.tree_state.active = Some(self.tree_state.tree.len() - 1);
        }
    }
    fn key_left(&mut self) {
        if let Some(i) = self.tree_state.active {
            if let Some(e) = self.tree_state.tree.get_mut(i) {
                if let Some(c) = e.children {
                    e.children = None;
                    self.key_left_i(i+1,i+1+c);
                } else if e.depth > 0 {
                    let d = e.depth;
                    let mut j = i;
                    while j > 0 {
                        j -= 1;
                        if self.tree_state.tree[j].depth < d {
                            self.tree_state.active = Some(j);
                            break;
                        }
                    }
                    if let Some(e) = self.tree_state.tree.get_mut(j) {
                        if let Some(c) = e.children {
                            e.children = None;
                            self.key_left_i(j+1,j+1+c);
                        }
                    }
                }
            }
        }
    }
    fn key_left_i(&mut self,from:usize,to:usize) {
        self.tree_state.tree.drain(from.. to);
    }
    fn key_right(&mut self,controller: &Controller) {
        if let Some(i) = self.tree_state.active {
            let r = if let Some(e) = self.tree_state.tree.get_mut(i) {
                let dp = e.depth + 1;
                let id = e.id.as_ref();
                match controller.archives().find(id.to_owned()) {
                    None => None,
                    Some(Either::Right(_)) => None, // TODO
                    Some(Either::Left(g)) => {
                        e.children = Some(g.base().archives.len());
                        Some((dp,g))
                    }
                }
            } else {None};
            if let Some((dp,g)) = r {
                for e in &g.base().archives {
                    self.tree_state.tree.insert(i+1,from_g(e,dp));
                }
            }
        }
    }
}
impl UITab for Library {
    fn handle_event(&mut self, controller: &Controller, event: Event) -> Result<(), Error> {
        if let Event::Key(key) = event { match key.code {
            KeyCode::Down => self.key_down(),
            KeyCode::Up => self.key_up(),
            KeyCode::Left => self.key_left(),
            KeyCode::Right => self.key_right(controller),
            /*
            KeyCode::Enter|KeyCode::Char(' ') => self.state.toggle_selected(),
            KeyCode::Home => {
                self.state.select_first(&self.items);
            }
            KeyCode::End => {
                self.state.select_last(&self.items);
            }
            KeyCode::PageDown => self.state.scroll_down(3),
            KeyCode::PageUp => self.state.scroll_up(3),

             */
            _ => ()
        }} else if let Event::Mouse(m) = event {
            match m.kind {
                MouseEventKind::ScrollDown => self.scroll_state.scroll_down(),
                MouseEventKind::ScrollUp => self.scroll_state.scroll_up(),
                _ => ()
            }
        }
        Ok(())
    }
    fn activate(&mut self, controller: &Controller) {
        if self.tree_state.tree.is_empty() {
            for e in &controller.archives().get_top().base().archives {
                self.tree_state.tree.push(from_g(e,0));
            }
        }
    }
    fn render(&mut self, area: Rect, buf: &mut Buffer) {
        //.highlight_symbol("⮕");
        let mut scroll_view = ScrollView::new(Size::new(area.width, self.tree_state.tree.len() as u16));
        self.treeview(scroll_view.buf_mut(),area.height);
        scroll_view.render(area, buf, &mut self.scroll_state);
    }
}

fn from_g(e:&Either<ArchiveGroup,Archive>,depth:u8) -> TreeElement {
    match e {
        Either::Right(a) => {
            TreeElement {
                id:a.id().to_owned(),
                render:format!("📕 {}",a.id().steps().last().unwrap()).into(),
                children:None,
                depth
            }
        },
        Either::Left(g) => {
            TreeElement {
                id:g.id().to_owned(),
                render:format!("📚 {}",g.id().steps().last().unwrap()).into(),
                children:None,
                depth
            }
        }
    }
}

struct TreeState {
    tree:Vec<TreeElement>,
    active:Option<usize>
}

impl TreeState {
    /*
    fn len(&self) -> usize {
        let mut curr = self.tree.iter();
        let mut stack = Vec::new();
        let mut len = 0;
        loop {
            if let Some(c) = curr.next() {
                len += 1;
                if !c.children.is_empty() {
                    let old = std::mem::replace(&mut curr, c.children.iter());
                    stack.push(old);
                }
            } else if let Some(next) = stack.pop() {
                curr = next;
            } else { break }
        }
        len
    }
    fn iter(&self) -> TreeIter<'_> {
        TreeIter {
            curr:self.tree.iter(),
            stack:Vec::new()
        }
    }
    fn iter_mut(&mut self) -> TreeIterMut<'_> {
        TreeIterMut {
            curr_v:&mut self.tree.as_mut_slice(),
            curr_i:0,
            stack:Vec::new()
        }
    }

     */
}

#[derive(PartialEq,Eq)]
struct TreeElement {
    id:ArchiveId,
    render:FinalStr,
    depth:u8,
    children:Option<usize>
}
impl PartialOrd for TreeElement {
    #[inline(always)]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for TreeElement {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

/*
struct TreeIter<'a> {
    curr:std::slice::Iter<'a,TreeElement>,
    stack:Vec<std::slice::Iter<'a,TreeElement>>
}
impl<'a> Iterator for TreeIter<'a> {
    type Item = &'a TreeElement;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(c) = self.curr.next() {
                if !c.children.is_empty() {
                    self.stack.push(std::mem::replace(&mut self.curr,c.children.iter()));
                }
                return Some(c);
            } else if let Some(next) = self.stack.pop() {
                self.curr = next;
            } else { break }
        }
        None
    }

}

struct TreeIterMut<'a> {
    curr_i:usize,
    curr_v:&'a mut &'a mut [TreeElement],
    stack:Vec<(usize,&'a mut &'a mut [TreeElement])>
}
impl<'a> Iterator for TreeIterMut<'a> {
    type Item = &'a mut TreeElement;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.curr_i < self.curr_v.len() {
                let c = &mut self.curr_v[self.curr_i];
                self.curr_i += 1;
                if !c.children.is_empty() {
                    self.stack.push((self.curr_i,std::mem::replace(&mut self.curr_v,&mut c.children.as_mut_slice())));
                    self.curr_i = 0;
                }
                return Some(c);
            } else if let Some((i,next)) = self.stack.pop() {
                self.curr_i = i;
                self.curr_v = next;
            } else { break }
        }
        None
    }
}

 */
