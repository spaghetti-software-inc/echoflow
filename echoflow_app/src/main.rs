use eframe::egui;

mod flowchart {
    use eframe::egui;
    use std::collections::HashMap;
    use std::io::Write;
    use std::process::{Command, Stdio};

    /// A node in the flow-chart.
    #[derive(Debug)]
    pub struct Node {
        pub id: usize,
        pub pos: egui::Pos2, // In world coordinates
        pub command: String,
        pub output: String,  // Intermediate result after running its command
    }

    /// A connection between two nodes.
    #[derive(Debug)]
    pub struct Connection {
        pub from: usize,
        pub to: usize,
    }

    /// Encapsulates the flow-chart UI.
    pub struct FlowChart {
        pub nodes: Vec<Node>,
        pub connections: Vec<Connection>,
        pub next_id: usize,
        pub selected_node: Option<usize>,
        pub connection_start: Option<usize>,

        /// How far the camera has been panned, in screen coordinates.
        pub pan_offset: egui::Vec2,
        /// Zoom factor (1.0 = 100%, 2.0 = 200%, etc.).
        pub zoom: f32,

        /// We only store the size (width & height) of the central panel,
        /// so we can compute the camera rectangle in world coordinates.
        pub main_view_rect_size: Option<egui::Vec2>,
    }

    impl Default for FlowChart {
        fn default() -> Self {
            Self {
                nodes: Vec::new(),
                connections: Vec::new(),
                next_id: 1,
                selected_node: None,
                connection_start: None,
                pan_offset: egui::Vec2::ZERO,
                zoom: 1.0,
                main_view_rect_size: None,
            }
        }
    }

    impl FlowChart {
        /// Add a new node at a default position.
        pub fn add_node(&mut self) {
            let node = Node {
                id: self.next_id,
                pos: egui::pos2(50.0, 50.0),
                command: format!("echo Node {}", self.next_id),
                output: String::new(),
            };
            self.next_id += 1;
            self.nodes.push(node);
        }
        
        /// Add a new node with a specific command.
        pub fn add_node_with_command(&mut self, command: &str) {
            let node = Node {
                id: self.next_id,
                pos: egui::pos2(50.0, 50.0), // You might adjust this to suit your needs.
                command: command.to_string(),
                output: String::new(),
            };
            self.next_id += 1;
            self.nodes.push(node);
        }
        
        
        /// Compute a linear chain of node IDs based on connections.
        /// Assumes a valid chain starts with a node having no incoming connection.
        pub fn get_pipeline_chain(&self) -> Option<Vec<usize>> {
            let mut incoming = HashMap::new();
            let mut outgoing = HashMap::new();
            for node in &self.nodes {
                incoming.insert(node.id, 0);
            }
            for conn in &self.connections {
                *incoming.entry(conn.to).or_insert(0) += 1;
                outgoing.insert(conn.from, conn.to);
            }
            let start_id = self
                .nodes
                .iter()
                .find(|n| incoming.get(&n.id) == Some(&0))?
                .id;
            let mut chain = vec![start_id];
            let mut current = start_id;
            while let Some(&next) = outgoing.get(&current) {
                chain.push(next);
                current = next;
            }
            Some(chain)
        }

        /// Runs the commands in sequence (piping each output into the next),
        /// and returns intermediate outputs for each command.
        pub fn run_pipeline_with_intermediates(
            &self,
            commands: &[String],
        ) -> Result<Vec<String>, String> {
            if commands.is_empty() {
                return Ok(vec![]);
            }
            let mut intermediate_outputs = Vec::new();

            // Run the first command:
            let output = Command::new("sh")
                .arg("-c")
                .arg(&commands[0])
                .output()
                .map_err(|e| format!("Failed to run command '{}': {}", commands[0], e))?;
            if !output.status.success() {
                return Err(String::from_utf8_lossy(&output.stderr).to_string());
            }
            let first_out = String::from_utf8_lossy(&output.stdout).to_string();
            intermediate_outputs.push(first_out.clone());
            let mut current_input = first_out;

            // Pipe subsequent commands:
            for command in commands.iter().skip(1) {
                let mut child = Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .spawn()
                    .map_err(|e| format!("Failed to run command '{}': {}", command, e))?;

                {
                    let child_stdin = child.stdin.as_mut().ok_or("Failed to open stdin")?;
                    child_stdin
                        .write_all(current_input.as_bytes())
                        .map_err(|e| format!("Failed to write to stdin: {}", e))?;
                }

                let output = child
                    .wait_with_output()
                    .map_err(|e| format!("Error waiting on command '{}': {}", command, e))?;
                if !output.status.success() {
                    return Err(String::from_utf8_lossy(&output.stderr).to_string());
                }
                let out_str = String::from_utf8_lossy(&output.stdout).to_string();
                intermediate_outputs.push(out_str.clone());
                current_input = out_str;
            }

            Ok(intermediate_outputs)
        }

        /// Draw the flow-chart in the main (central) panel.
        /// This captures the panel size, handles pan/zoom, draws nodes, etc.
        pub fn draw(&mut self, ui: &mut egui::Ui) {
            // 1) Store just the size (width & height) of the central panel:
            let panel_size = ui.available_size_before_wrap();
            self.main_view_rect_size = Some(panel_size);

            // 2) Process mouse wheel for zoom:
            let scroll_delta = ui.input(|i| i.raw_scroll_delta);
            if scroll_delta.y != 0.0 {
                let zoom_factor = 1.0 + scroll_delta.y * 0.001;
                self.zoom *= zoom_factor;
            }

            // We'll do a simple world->screen transform:
            let transform = |world: egui::Pos2| -> egui::Pos2 {
                world * self.zoom + self.pan_offset
            };

            // Node drawing:
            let node_size = egui::vec2(120.0, 50.0) * self.zoom;
            let mut node_rects = std::collections::HashMap::new();

            // Allocate rects for nodes:
            for node in &mut self.nodes {
                let screen_pos = transform(node.pos);
                let rect = egui::Rect::from_min_size(screen_pos, node_size);
                let response = ui.allocate_rect(rect, egui::Sense::click_and_drag());
                if response.dragged() {
                    // Convert drag delta from screen coords to world coords:
                    node.pos += response.drag_delta() / self.zoom;
                }
                if response.clicked() {
                    self.selected_node = Some(node.id);
                }
                node_rects.insert(node.id, rect);
            }

            // Connection handles:
            let handle_size = egui::vec2(10.0, 10.0) * self.zoom;
            for (id, rect) in &node_rects {
                let handle_pos = egui::pos2(
                    rect.max.x - handle_size.x / 2.0,
                    rect.center().y - handle_size.y / 2.0,
                );
                let handle_rect = egui::Rect::from_min_size(handle_pos, handle_size);

                let handle_response =
                    ui.interact(handle_rect, egui::Id::new(*id), egui::Sense::click());
                if handle_response.clicked() {
                    if self.connection_start.is_none() {
                        self.connection_start = Some(*id);
                    } else if let Some(start_id) = self.connection_start {
                        if start_id != *id {
                            self.connections.push(Connection {
                                from: start_id,
                                to: *id,
                            });
                        }
                        self.connection_start = None;
                    }
                }

                ui.painter()
                    .rect_filled(handle_rect, 2.0, egui::Color32::YELLOW);
            }

            // Temporary connection line if the user is dragging from a node handle:
            if let Some(start_id) = self.connection_start {
                if let Some(&start_rect) = node_rects.get(&start_id) {
                    let start_handle = egui::pos2(start_rect.max.x, start_rect.center().y);
                    let pointer_pos = ui
                        .input(|i| i.pointer.hover_pos())
                        .unwrap_or(start_handle);
                    ui.painter().line_segment(
                        [start_handle, pointer_pos],
                        egui::Stroke::new(2.0, egui::Color32::RED),
                    );
                }
            }

            // Draw established connections:
            for conn in &self.connections {
                if let (Some(&from_rect), Some(&to_rect)) =
                    (node_rects.get(&conn.from), node_rects.get(&conn.to))
                {
                    ui.painter().line_segment(
                        [from_rect.center(), to_rect.center()],
                        egui::Stroke::new(2.0, egui::Color32::LIGHT_GREEN),
                    );
                }
            }

            // Finally, draw each node's background + text:
            for node in &self.nodes {
                if let Some(&rect) = node_rects.get(&node.id) {
                    let is_selected = Some(node.id) == self.selected_node;
                    let fill_color = egui::Color32::from_rgb(100, 150, 200);
                    let stroke = if is_selected {
                        egui::Stroke::new(3.0, egui::Color32::GOLD)
                    } else {
                        egui::Stroke::new(2.0, egui::Color32::BLACK)
                    };

                    ui.painter().rect_filled(rect, 5.0, fill_color);
                    ui.painter().rect_stroke(rect, 5.0, stroke);
                    ui.painter().text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        &node.command,
                        egui::FontId::proportional(16.0 * self.zoom),
                        egui::Color32::WHITE,
                    );
                }
            }
        }

        /// Draw a minimap in the given UI, showing nodes, connections, and a red camera rectangle.
        pub fn draw_minimap(&self, ui: &mut egui::Ui) {
            let mut min = egui::pos2(f32::INFINITY, f32::INFINITY);
            let mut max = egui::pos2(f32::NEG_INFINITY, f32::NEG_INFINITY);
            for node in &self.nodes {
                min.x = min.x.min(node.pos.x);
                min.y = min.y.min(node.pos.y);
                max.x = max.x.max(node.pos.x);
                max.y = max.y.max(node.pos.y);
            }
            let padding = egui::vec2(50.0, 50.0);
            min -= padding;
            max += padding;
            let world_rect = egui::Rect::from_min_max(min, max);

            let minimap_size = egui::vec2(200.0, 150.0);
            let minimap_rect =
                egui::Rect::from_min_size(ui.min_rect().min + egui::vec2(10.0, 10.0), minimap_size);
            ui.painter()
                .rect_filled(minimap_rect, 3.0, egui::Color32::DARK_GRAY);

            let world_size = world_rect.size();
            let scale_x = minimap_size.x / world_size.x;
            let scale_y = minimap_size.y / world_size.y;
            let minimap_scale = scale_x.min(scale_y);
            let extra_space = minimap_size - world_size * minimap_scale;
            let offset = extra_space * 0.5;

            for node in &self.nodes {
                let minimap_pos =
                    minimap_rect.min + offset + (node.pos - world_rect.min) * minimap_scale;
                let node_size = egui::vec2(20.0, 10.0);
                let node_rect = egui::Rect::from_center_size(minimap_pos, node_size);
                ui.painter().rect_filled(node_rect, 2.0, egui::Color32::LIGHT_BLUE);
            }

            for conn in &self.connections {
                let from_node = self.nodes.iter().find(|n| n.id == conn.from);
                let to_node = self.nodes.iter().find(|n| n.id == conn.to);
                if let (Some(from), Some(to)) = (from_node, to_node) {
                    let from_minimap =
                        minimap_rect.min + offset + (from.pos - world_rect.min) * minimap_scale;
                    let to_minimap =
                        minimap_rect.min + offset + (to.pos - world_rect.min) * minimap_scale;
                    ui.painter().line_segment(
                        [from_minimap, to_minimap],
                        egui::Stroke::new(1.0, egui::Color32::WHITE),
                    );
                }
            }

            if let Some(panel_size) = self.main_view_rect_size {
                let camera_min_vec = -self.pan_offset / self.zoom;
                let camera_max_vec = camera_min_vec + (panel_size / self.zoom);
                let mut camera_rect = egui::Rect::from_min_max(
                    egui::pos2(camera_min_vec.x, camera_min_vec.y),
                    egui::pos2(camera_max_vec.x, camera_max_vec.y),
                );
                camera_rect = camera_rect.intersect(world_rect);

                let minimap_viewport_min = minimap_rect.min
                    + offset
                    + (camera_rect.min - world_rect.min) * minimap_scale;
                let minimap_viewport_max = minimap_rect.min
                    + offset
                    + (camera_rect.max - world_rect.min) * minimap_scale;
                let minimap_viewport =
                    egui::Rect::from_min_max(minimap_viewport_min, minimap_viewport_max);

                ui.painter().rect_stroke(
                    minimap_viewport,
                    2.0,
                    egui::Stroke::new(1.0, egui::Color32::RED),
                );
            }
        }
    }
}

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
        if ctx.input(|i| i.key_pressed(egui::Key::Plus)) {
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
        "Pipeline Flowchart UI with Global Controls",
        native_options,
        Box::new(|_cc| Box::new(PipelineApp::default())),
    );
}
