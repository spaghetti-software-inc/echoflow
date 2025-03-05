#[derive(Debug)]
pub enum FlowChartCommand {
    AddNode,
    RunPipeline,
    DeleteSelectedNode,
    PanLeft,
    PanRight,
    PanUp,
    PanDown,
    ZoomIn,
    ZoomOut,
} 