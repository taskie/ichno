pub const DEFAULT_NAMESPACE_ID: &'static str = "default";
pub const META_NAMESPACE_ID: &'static str = "__meta";

pub enum Status {
    DISABLED = 0,
    ENABLED = 1,
}

pub enum NamespaceType {
    LOCAL = 0,
    REMOTE = 1,
}
