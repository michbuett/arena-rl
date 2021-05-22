
pub struct Visual {
    elements: Vec<VisualElement>
}

pub enum VisualElement {
    Static(String),
    SimpleAnimation(String),
}
