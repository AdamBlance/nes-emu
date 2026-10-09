struct VisibleScanline(u16);
impl VisibleScanline {
    const fn new(line: u16) -> Self {
        assert!(line < 240);
        Self(line)
    }
}

struct BlankingScanline(u16);
impl BlankingScanline {
    const fn new(line: u16) -> Self {
        assert!(line >= 240 && line <= 260);
        Self(line)
    }
}

struct Dot(u16);
impl Dot {
    const fn new(dot: u16) -> Self {
        assert!(dot <= 340);
        Self(dot)
    }
}

enum PpuState {
    Rendering(VisibleScanline, Dot),
    VBlank(BlankingScanline, Dot),
    PreRenderScanline(Dot)
}


struct PpuStateMachine {

}