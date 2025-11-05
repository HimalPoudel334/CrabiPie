use eframe::egui;
use std::sync::mpsc::{self, Receiver, Sender};

#[derive(PartialEq)]
enum RequestTab {
    Body,
    Headers,
}

struct MyApp {
    // Request configuration
    url: String,
    method: HttpMethod,
    headers: String,
    body: String,

    // Response data
    response_status: String,
    response_headers: String,
    response_body: String,

    // UI state
    loading: bool,
    active_request_tab: RequestTab,
    active_response_tab: ResponseTab,

    // Communication channel for async requests
    tx: Sender<HttpResponse>,
    rx: Receiver<HttpResponse>,
}

#[derive(PartialEq, Clone, Debug)]
enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
}

#[derive(PartialEq)]
enum ResponseTab {
    Body,
    Headers,
}

struct HttpResponse {
    status: String,
    headers: String,
    body: String,
}

impl Default for MyApp {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            url: "http://202.51.1.29:8200/localmediprowebapi/".to_string(),
            method: HttpMethod::GET,
            headers: String::new(),
            body: r#"{
  "title": "foo",
  "body": "bar",
  "userId": 1
}"#
            .to_string(),
            response_status: String::new(),
            response_headers: String::new(),
            response_body: String::new(),
            loading: false,
            active_request_tab: RequestTab::Body,
            active_response_tab: ResponseTab::Body,
            tx,
            rx,
        }
    }
}

impl MyApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self::default()
    }

    fn name() -> &'static str {
        "CrabPie"
    }

    fn send_request(&mut self) {
        let url = self.url.clone();
        if url.is_empty() {
            self.response_body = "Error! Please enter a valid url!!".to_string();
            return;
        }

        self.loading = true;
        self.response_body = "Loading...".to_string();
        self.response_status = String::new();

        let method = self.method.clone();
        let body = self.body.clone();
        let tx = self.tx.clone();

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let response = rt.block_on(async {
                let client = reqwest::Client::new();

                let request = match method {
                    HttpMethod::GET => client.get(&url),
                    HttpMethod::POST => client
                        .post(&url)
                        .body(body)
                        .header("Content-Type", "application/json"),
                    HttpMethod::PUT => client
                        .put(&url)
                        .body(body)
                        .header("Content-Type", "application/json"),
                    HttpMethod::DELETE => client.delete(&url),
                    HttpMethod::PATCH => client
                        .patch(&url)
                        .body(body)
                        .header("Content-Type", "application/json"),
                };

                match request.send().await {
                    Ok(resp) => {
                        let status = format!(
                            "{} {}",
                            resp.status().as_u16(),
                            resp.status().canonical_reason().unwrap_or("")
                        );
                        let headers = format!("{:#?}", resp.headers());
                        let body_text = resp
                            .text()
                            .await
                            .unwrap_or_else(|e| format!("Error reading body: {}", e));

                        // Try to pretty print JSON
                        let body = if let Ok(json) =
                            serde_json::from_str::<serde_json::Value>(&body_text)
                        {
                            serde_json::to_string_pretty(&json).unwrap_or(body_text)
                        } else {
                            body_text
                        };

                        HttpResponse {
                            status,
                            headers,
                            body,
                        }
                    }
                    Err(e) => HttpResponse {
                        status: "Error".to_string(),
                        headers: String::new(),
                        body: format!("Request failed: {}", e),
                    },
                }
            });

            let _ = tx.send(response);
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size((900.0, 600.0))
            .with_min_inner_size((800.0, 500.0)),
        ..eframe::NativeOptions::default()
    };
    eframe::run_native(
        MyApp::name(),
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_pixels_per_point(1.5);

        // Check for response from background thread
        if let Ok(resp) = self.rx.try_recv() {
            self.response_status = resp.status;
            self.response_headers = resp.headers;
            self.response_body = resp.body;
            self.loading = false;
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("CrabPie HTTP Client");
            ui.add_space(10.0);

            // Request section
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    // Method dropdown
                    egui::ComboBox::from_id_salt("method")
                        .selected_text(format!("{:?}", self.method))
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut self.method, HttpMethod::GET, "GET");
                            ui.selectable_value(&mut self.method, HttpMethod::POST, "POST");
                            ui.selectable_value(&mut self.method, HttpMethod::PUT, "PUT");
                            ui.selectable_value(&mut self.method, HttpMethod::DELETE, "DELETE");
                            ui.selectable_value(&mut self.method, HttpMethod::PATCH, "PATCH");
                        });

                    // URL input - take all remaining space except for Send button
                    let available_width = ui.available_width() - 55.0;
                    ui.add(
                        egui::TextEdit::singleline(&mut self.url).desired_width(available_width),
                    );

                    // Send button
                    if ui
                        .add_enabled(!self.loading, egui::Button::new("Send"))
                        .clicked()
                    {
                        self.send_request();
                    }
                });
            });

            ui.add_space(10.0);

            // Request tabs (only show for methods that can have a body)
            if matches!(
                self.method,
                HttpMethod::POST | HttpMethod::PUT | HttpMethod::PATCH
            ) {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.active_request_tab, RequestTab::Body, "Body");
                    ui.selectable_value(
                        &mut self.active_request_tab,
                        RequestTab::Headers,
                        "Headers",
                    );
                });

                ui.separator();

                // Request body/headers editor
                egui::ScrollArea::vertical()
                    .id_salt("request_scroll")
                    .max_height(200.0)
                    .show(ui, |ui| match self.active_request_tab {
                        RequestTab::Body => {
                            egui::TextEdit::multiline(&mut self.body)
                                .desired_width(f32::INFINITY)
                                .desired_rows(8)
                                .code_editor()
                                .show(ui);
                        }
                        RequestTab::Headers => {
                            egui::TextEdit::multiline(&mut self.headers)
                                .desired_width(f32::INFINITY)
                                .desired_rows(8)
                                .code_editor()
                                .show(ui);
                        }
                    });

                ui.add_space(10.0);
            }

            // Response section label
            ui.label(egui::RichText::new("Response").strong());

            // Response tabs
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_response_tab, ResponseTab::Body, "Body");
                ui.selectable_value(
                    &mut self.active_response_tab,
                    ResponseTab::Headers,
                    "Headers",
                );

                if !self.response_status.is_empty() {
                    ui.separator();
                    ui.label(&self.response_status);
                }

                if self.loading {
                    ui.spinner();
                }
            });

            ui.separator();

            // Response display
            egui::ScrollArea::vertical()
                .id_salt("response_scroll")
                .show(ui, |ui| match self.active_response_tab {
                    ResponseTab::Body => {
                        egui::TextEdit::multiline(&mut self.response_body.as_str())
                            .desired_width(f32::INFINITY)
                            .desired_rows(20)
                            .code_editor()
                            .show(ui);
                    }
                    ResponseTab::Headers => {
                        egui::TextEdit::multiline(&mut self.response_headers.as_str())
                            .desired_width(f32::INFINITY)
                            .desired_rows(20)
                            .code_editor()
                            .show(ui);
                    }
                });
        });

        // Keep repainting while loading
        if self.loading {
            ctx.request_repaint();
        }
    }
}
