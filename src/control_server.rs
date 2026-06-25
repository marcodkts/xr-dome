use std::thread;

use tiny_http::{
    Header,
    Method,
    Response,
    Server,
    StatusCode,
};

use crate::dome_config::{
    DomeConfig,
    SharedDomeConfig,
};

const INDEX_HTML: &str = r#"
<!doctype html>
<html lang="pt-BR">
<head>
    <meta charset="utf-8" />
    <title>XR Dome Controls</title>
    <style>
        body {
            margin: 0;
            padding: 18px;
            background: #0b1020;
            color: #e8ecff;
            font-family: system-ui, sans-serif;
        }

        h1 {
            margin: 0 0 14px;
            font-size: 18px;
        }

        .row {
            margin-bottom: 14px;
        }

        label {
            display: flex;
            justify-content: space-between;
            font-size: 13px;
            margin-bottom: 6px;
        }

        input[type="range"] {
            width: 100%;
        }

        .buttons {
            display: grid;
            grid-template-columns: 1fr 1fr;
            gap: 8px;
            margin-top: 18px;
        }

        button {
            background: #1d2a4a;
            color: #e8ecff;
            border: 1px solid #34466f;
            border-radius: 8px;
            padding: 9px;
            cursor: pointer;
        }

        button:hover {
            background: #293b66;
        }

        .hint {
            margin-top: 16px;
            color: #9da8c7;
            font-size: 12px;
            line-height: 1.35;
        }
    </style>
</head>
<body>
    <h1>XR Dome Controls</h1>

    <div class="row">
        <label>Raio <span id="radius_value"></span></label>
        <input id="radius" type="range" min="1" max="10" step="0.1">
    </div>

    <div class="row">
        <label>Abertura horizontal / yaw <span id="yaw_degrees_value"></span></label>
        <input id="yaw_degrees" type="range" min="30" max="360" step="1">
    </div>

    <div class="row">
        <label>Pitch mínimo <span id="min_pitch_degrees_value"></span></label>
        <input id="min_pitch_degrees" type="range" min="-89" max="0" step="1">
    </div>

    <div class="row">
        <label>Pitch máximo <span id="max_pitch_degrees_value"></span></label>
        <input id="max_pitch_degrees" type="range" min="0" max="89" step="1">
    </div>

    <div class="row">
        <label>Segmentos horizontais <span id="horizontal_segments_value"></span></label>
        <input id="horizontal_segments" type="range" min="32" max="1024" step="16">
    </div>

    <div class="row">
        <label>Segmentos verticais <span id="vertical_segments_value"></span></label>
        <input id="vertical_segments" type="range" min="8" max="256" step="8">
    </div>

    <div class="buttons">
        <button onclick="presetWorkArea()">Área útil</button>
        <button onclick="presetDome()">Domo XR</button>
        <button onclick="presetFullSphere()">Quase esfera</button>
        <button onclick="presetLight()">Leve</button>
    </div>

    <div class="hint">
        Use este painel apenas para calibrar o espaço. A área real de trabalho
        deve ficar principalmente entre -45° e +45° de pitch.
    </div>

    <script>
        const fields = [
            "radius",
            "yaw_degrees",
            "min_pitch_degrees",
            "max_pitch_degrees",
            "horizontal_segments",
            "vertical_segments",
        ];

        let syncTimer = null;

        function readForm() {
            return {
                radius: parseFloat(radius.value),
                yaw_degrees: parseFloat(yaw_degrees.value),
                min_pitch_degrees: parseFloat(min_pitch_degrees.value),
                max_pitch_degrees: parseFloat(max_pitch_degrees.value),
                horizontal_segments: parseInt(horizontal_segments.value),
                vertical_segments: parseInt(vertical_segments.value),
            };
        }

        function writeForm(cfg) {
            for (const field of fields) {
                document.getElementById(field).value = cfg[field];
                document.getElementById(field + "_value").innerText = cfg[field];
            }
        }

        async function sync() {
            const cfg = readForm();

            writeForm(cfg);

            await fetch("/config", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify(cfg),
            });
        }

        function scheduleSync() {
            clearTimeout(syncTimer);
            syncTimer = setTimeout(sync, 100);
        }

        async function load() {
            const response = await fetch("/config");
            const cfg = await response.json();

            writeForm(cfg);
        }

        function presetWorkArea() {
            writeForm({
                radius: 3.2,
                yaw_degrees: 240,
                min_pitch_degrees: -45,
                max_pitch_degrees: 45,
                horizontal_segments: 384,
                vertical_segments: 96,
            });

            sync();
        }

        function presetDome() {
            writeForm({
                radius: 3.2,
                yaw_degrees: 360,
                min_pitch_degrees: -75,
                max_pitch_degrees: 75,
                horizontal_segments: 512,
                vertical_segments: 128,
            });

            sync();
        }

        function presetFullSphere() {
            writeForm({
                radius: 3.2,
                yaw_degrees: 360,
                min_pitch_degrees: -85,
                max_pitch_degrees: 85,
                horizontal_segments: 512,
                vertical_segments: 160,
            });

            sync();
        }

        function presetLight() {
            writeForm({
                radius: 3.2,
                yaw_degrees: 360,
                min_pitch_degrees: -70,
                max_pitch_degrees: 70,
                horizontal_segments: 256,
                vertical_segments: 64,
            });

            sync();
        }

        for (const field of fields) {
            document.getElementById(field).addEventListener("input", scheduleSync);
        }

        load();
    </script>
</body>
</html>
"#;

pub fn spawn_control_server(shared_config: SharedDomeConfig) {
    thread::Builder::new()
        .name("xr-dome-control-server".to_string())
        .spawn(move || {
            let address = "127.0.0.1:3760";

            let server = match Server::http(address) {
                Ok(server) => server,
                Err(error) => {
                    eprintln!("control server failed: {error}");
                    return;
                }
            };

            println!("XR Dome control panel: http://{address}");

            for mut request in server.incoming_requests() {
                let method = request.method().clone();
                let url = request.url().to_string();

                match (method, url.as_str()) {
                    (Method::Get, "/") => {
                        let _ = request.respond(html_response(INDEX_HTML));
                    }

                    (Method::Get, "/config") => {
                        let config = shared_config.get();

                        match serde_json::to_string_pretty(&config) {
                            Ok(body) => {
                                let _ = request.respond(json_response(StatusCode(200), body));
                            }

                            Err(error) => {
                                let _ = request.respond(text_response(
                                    StatusCode(500),
                                    format!("json error: {error}"),
                                ));
                            }
                        }
                    }

                    (Method::Post, "/config") => {
                        let mut body = String::new();

                        if let Err(error) = request.as_reader().read_to_string(&mut body) {
                            let _ = request.respond(text_response(
                                StatusCode(400),
                                format!("read error: {error}"),
                            ));

                            continue;
                        }

                        match serde_json::from_str::<DomeConfig>(&body) {
                            Ok(config) => {
                                shared_config.set(config);

                                let _ = request.respond(json_response(
                                    StatusCode(200),
                                    "{\"ok\":true}".to_string(),
                                ));
                            }

                            Err(error) => {
                                let _ = request.respond(text_response(
                                    StatusCode(400),
                                    format!("invalid config: {error}"),
                                ));
                            }
                        }
                    }

                    _ => {
                        let _ = request.respond(text_response(
                            StatusCode(404),
                            "not found".to_string(),
                        ));
                    }
                }
            }
        })
        .expect("failed to spawn control server");
}

fn html_response(body: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body)
        .with_status_code(StatusCode(200))
        .with_header(
            Header::from_bytes(
                &b"Content-Type"[..],
                &b"text/html; charset=utf-8"[..],
            )
            .unwrap(),
        )
}

fn json_response(
    status: StatusCode,
    body: String,
) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body)
        .with_status_code(status)
        .with_header(
            Header::from_bytes(
                &b"Content-Type"[..],
                &b"application/json; charset=utf-8"[..],
            )
            .unwrap(),
        )
}

fn text_response(
    status: StatusCode,
    body: String,
) -> Response<std::io::Cursor<Vec<u8>>> {
    Response::from_string(body)
        .with_status_code(status)
        .with_header(
            Header::from_bytes(
                &b"Content-Type"[..],
                &b"text/plain; charset=utf-8"[..],
            )
            .unwrap(),
        )
}