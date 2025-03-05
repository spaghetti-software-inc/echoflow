use crate::commands::FlowChartCommand;
use crate::flowchart::FlowChart;
use eframe::egui;

pub struct PipelineApp {
    pub flowchart: FlowChart,
    pub pipeline_output: String,
}

impl Default for PipelineApp {
    fn default() -> Self {
        Self {
            flowchart: FlowChart::default(),
            pipeline_output: String::new(),
        }
    }
}

impl PipelineApp {
    /// Executes a given command that affects the flowchart.
    pub fn execute_command(&mut self, command: FlowChartCommand) {
        match command {
            FlowChartCommand::AddNode => {
                self.flowchart.add_node();
            }
            FlowChartCommand::RunPipeline => {
                if let Some(chain) = self.flowchart.get_pipeline_chain() {
                    let commands: Vec<String> = chain
                        .iter()
                        .filter_map(|id| self.flowchart.nodes.iter().find(|n| n.id == *id))
                        .map(|node| node.command.clone())
                        .collect();
                    match self.flowchart.run_pipeline_with_intermediates(&commands) {
                        Ok(outputs) => {
                            for (i, id) in chain.iter().enumerate() {
                                if let Some(node) =
                                    self.flowchart.nodes.iter_mut().find(|n| n.id == *id)
                                {
                                    node.output = outputs.get(i).cloned().unwrap_or_default();
                                }
                            }
                            self.pipeline_output = outputs.last().cloned().unwrap_or_default();
                        }
                        Err(e) => self.pipeline_output = e,
                    }
                } else {
                    self.pipeline_output = "No valid pipeline chain found.".into();
                }
            }
            FlowChartCommand::DeleteSelectedNode => {
                if let Some(selected_id) = self.flowchart.selected_node {
                    self.flowchart.nodes.retain(|node| node.id != selected_id);
                    self.flowchart.connections.retain(|conn| {
                        conn.from != selected_id && conn.to != selected_id
                    });
                    if self.flowchart.connection_start == Some(selected_id) {
                        self.flowchart.connection_start = None;
                    }
                    self.flowchart.selected_node = None;
                }
            }
            FlowChartCommand::PanLeft => {
                self.flowchart.pan_offset.x += 20.0;
            }
            FlowChartCommand::PanRight => {
                self.flowchart.pan_offset.x -= 20.0;
            }
            FlowChartCommand::PanUp => {
                self.flowchart.pan_offset.y += 20.0;
            }
            FlowChartCommand::PanDown => {
                self.flowchart.pan_offset.y -= 20.0;
            }
            FlowChartCommand::ZoomIn => {
                self.flowchart.zoom *= 1.1;
            }
            FlowChartCommand::ZoomOut => {
                self.flowchart.zoom /= 1.1;
            }
        }
    }
} 