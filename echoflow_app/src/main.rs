use eframe::egui;

mod flowchart;

// Define an enum to list all global commands.
#[derive(Debug)]
enum FlowChartCommand {
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

struct PipelineApp {
    flowchart: flowchart::FlowChart,
    pipeline_output: String, // Final output from the pipeline
}

impl Default for PipelineApp {
    fn default() -> Self {
        Self {
            flowchart: flowchart::FlowChart::default(),
            pipeline_output: String::new(),
        }
    }
}

impl PipelineApp {
    /// Executes a given command that affects the flowchart.
    fn execute_command(&mut self, command: FlowChartCommand) {
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

impl eframe::App for PipelineApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Reset connection mode when the Escape key is pressed.
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.flowchart.connection_start = None;
        }
        
        // Toolbox panel on the left:
        egui::SidePanel::left("toolbox_panel").show(ctx, |ui| {
            ui.heading("Toolbox");
            let presets = vec![
                ("Echo", "echo Hello World"),
                ("List Directory", "ls -la"),
                ("Grep", "grep 'pattern'"),
                ("Sort", "sort"),
                ("Word Count", "wc -w"),
            ];
            for (name, command) in presets {
                if ui.button(name).clicked() {
                    self.flowchart.add_node_with_command(command);
                }
            }
        });        
        
        // Top panel with UI buttons that trigger global controls.
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Add Node").clicked() {
                    self.execute_command(FlowChartCommand::AddNode);
                }
                if ui.button("Run Pipeline").clicked() {
                    self.execute_command(FlowChartCommand::RunPipeline);
                }
                if ui.button("Delete Selected Node").clicked() {
                    self.execute_command(FlowChartCommand::DeleteSelectedNode);
                }
                if ui.button("Pan Left").clicked() {
                    self.execute_command(FlowChartCommand::PanLeft);
                }
                if ui.button("Pan Right").clicked() {
                    self.execute_command(FlowChartCommand::PanRight);
                }
                if ui.button("Pan Up").clicked() {
                    self.execute_command(FlowChartCommand::PanUp);
                }
                if ui.button("Pan Down").clicked() {
                    self.execute_command(FlowChartCommand::PanDown);
                }
                if ui.button("Zoom In").clicked() {
                    self.execute_command(FlowChartCommand::ZoomIn);
                }
                if ui.button("Zoom Out").clicked() {
                    self.execute_command(FlowChartCommand::ZoomOut);
                }
            });
        });

        // Keyboard shortcuts for global controls.
        if ctx.input(|i| i.key_pressed(egui::Key::N)) {
            self.execute_command(FlowChartCommand::AddNode);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::R)) {
            self.execute_command(FlowChartCommand::RunPipeline);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
            self.execute_command(FlowChartCommand::DeleteSelectedNode);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowLeft)) {
            self.execute_command(FlowChartCommand::PanLeft);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowRight)) {
            self.execute_command(FlowChartCommand::PanRight);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowUp)) {
            self.execute_command(FlowChartCommand::PanUp);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
            self.execute_command(FlowChartCommand::PanDown);
        }
        // Note: Depending on your keyboard/layout, you might need to adjust these key choices.
        if ctx.input(|i| i.key_pressed(egui::Key::Equals)) {
            self.execute_command(FlowChartCommand::ZoomIn);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Minus)) {
            self.execute_command(FlowChartCommand::ZoomOut);
        }

        // Side panel: show details for the selected node.
        if let Some(selected_id) = self.flowchart.selected_node {
            egui::SidePanel::right("side_panel").show(ctx, |ui| {
                if let Some(node) = self
                    .flowchart
                    .nodes
                    .iter_mut()
                    .find(|n| n.id == selected_id)
                {
                    ui.heading(format!("Node {}", node.id));
                    ui.label("Command:");
                    ui.text_edit_singleline(&mut node.command);
                    ui.separator();
                    ui.label("Intermediate Output:");
                    ui.code(&node.output);
                }
            });
        }

        // Bottom panel: display final pipeline output.
        egui::TopBottomPanel::bottom("output_panel").show(ctx, |ui| {
            ui.heading("Pipeline Final Output");
            ui.label(&self.pipeline_output);
        });

        // Central panel: draw the flow-chart.
        egui::CentralPanel::default().show(ctx, |ui| {
            self.flowchart.draw(ui);
        });

        // Minimap overlay.
        egui::Area::new("minimap".into())
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
            .show(ctx, |ui| {
                ui.set_min_size(egui::vec2(220.0, 170.0));
                self.flowchart.draw_minimap(ui);
            });
    }
}

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Echoflow v0.0.1",
        native_options,
        Box::new(|_cc| Box::new(PipelineApp::default())),
    );
}
