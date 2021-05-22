use sdl2::event::Event as Sdl2Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

use crate::core::*;
use crate::ui::{ClickArea, ClickAreas, ScreenPos, UI};

pub fn poll(sdl_events: &mut EventPump, click_areas: &ClickAreas, ui: &UI) -> Option<UserInput> {
    for event in sdl_events.poll_iter() {
        match event {
            Sdl2Event::Quit { .. }
            | Sdl2Event::KeyDown {
                keycode: Some(Keycode::Escape),
                ..
            } => return Some(UserInput::Exit()),

            Sdl2Event::MouseMotion { xrel, yrel, .. } => {
                let is_scrolling = ui.scrolling.as_ref().map(|s| s.is_scrolling).unwrap_or(false);
                if is_scrolling {
                    return Some(UserInput::ScrollTo(
                        ui.pixel_ratio as i32 * xrel,
                        ui.pixel_ratio as i32 * yrel,
                    ));
                }
            }

            Sdl2Event::MouseButtonUp { x, y, .. } => {
                let has_scrolled = ui.scrolling.as_ref().map(|s| s.has_scrolled).unwrap_or(false);
                if has_scrolled {
                    return Some(UserInput::EndScrolling());
                } else {
                    let p = ScreenPos(ui.pixel_ratio as i32 * x, ui.pixel_ratio as i32 * y); 

                    for ClickArea {clipping_area, action,} in click_areas.iter() {
                        if clipping_area.contains_point(p.to_point()) {
                            return Some(action(p));
                        }
                    }
                }
            }

            Sdl2Event::MouseButtonDown { .. } => {
                return Some(UserInput::StartScrolling());
            }

            _ => {}
        }
    }

    None
}
