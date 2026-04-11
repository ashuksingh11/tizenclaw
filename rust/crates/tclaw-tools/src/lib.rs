use tclaw_api::SurfaceDescriptor;

pub fn tool_surface() -> SurfaceDescriptor {
    SurfaceDescriptor {
        name: "tools".into(),
        role: "tool integration boundary".into(),
    }
}
