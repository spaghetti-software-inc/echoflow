use crate::app::PipelineApp;
use crate::commands::FlowChartCommand;
use eframe::egui;

impl eframe::App for PipelineApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Reset connection mode when the Escape key is pressed.
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.flowchart.connection_start = None;
        }
        
        self.draw_toolbox_panel(ctx);
        self.draw_top_panel(ctx);
        self.handle_keyboard_shortcuts(ctx);
        self.draw_side_panel(ctx);
        self.draw_bottom_panel(ctx);
        self.draw_central_panel(ctx);
        self.draw_minimap(ctx);
    }
}

impl PipelineApp {
    fn draw_toolbox_panel(&mut self, ctx: &egui::Context) {
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
    }

    fn draw_top_panel(&mut self, ctx: &egui::Context) {
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
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
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
        if ctx.input(|i| i.key_pressed(egui::Key::Equals)) {
            self.execute_command(FlowChartCommand::ZoomIn);
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Minus)) {
            self.execute_command(FlowChartCommand::ZoomOut);
        }
    }

    fn draw_side_panel(&mut self, ctx: &egui::Context) {
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
    }

    fn draw_bottom_panel(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("output_panel").show(ctx, |ui| {
            ui.heading("Pipeline Final Output");
            ui.label(&self.pipeline_output);
        });
    }

    fn draw_central_panel(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.flowchart.draw(ui);
        });
    }

    fn draw_minimap(&mut self, ctx: &egui::Context) {
        egui::Area::new("minimap".into())
            .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
            .show(ctx, |ui| {
                ui.set_min_size(egui::vec2(220.0, 170.0));
                self.flowchart.draw_minimap(ui);
            });
    }
} 