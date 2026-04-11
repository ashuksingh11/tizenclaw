use tclaw_api::SurfaceDescriptor;

pub fn plugin_surface() -> SurfaceDescriptor {
    SurfaceDescriptor {
        name: "plugins".into(),
        role: "plugin loading boundary".into(),
    }
}
