pub trait Widget {
    type Builder;
}

#[typetag::deserialize(tag = "widget", content = "properties")]
pub trait WidgetBuilder: std::fmt::Debug {}
