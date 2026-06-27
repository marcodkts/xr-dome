# XR Dome

Desktop XR experimental para Linux.

## Executar

Depois de instalar a regra `udev` da VITURE, o comando normal fica:

```bash
cargo run
```

O app lê `.env` na inicialização. Você pode copiar [`.env.example`](./.env.example) para `.env` e ajustar as variáveis assim:

```bash
VITURE_SDK_ROOT=./viture_sdk
VITURE_LOG_IMU=1
RUST_LOG=xr_dome=info
```

Os valores mais úteis para o campo de visão e a distância virtual são:

```bash
XR_DOME_OBSERVER_DISTANCE_M=2.5
XR_DOME_DOME_YAW_DEGREES=140
XR_DOME_DOME_MIN_PITCH_DEGREES=-30
XR_DOME_DOME_MAX_PITCH_DEGREES=30
XR_DOME_WORKSPACE_YAW_DEGREES=140
XR_DOME_WORKSPACE_PITCH_DEGREES=60
XR_DOME_WORKSPACE_TITLE=XR Desktop
XR_DOME_WORKSPACE_SUBTITLE=Workspace 3DoF em arco frontal
```

## Permissões da VITURE

O SDK acessa o hardware via USB e `hidraw`, então o óculos precisa de uma regra `udev`.
Isso evita rodar o app com `sudo` toda vez.

O VID/PID do VITURE Pro detectado neste ambiente foi `35ca:101d`.

Instale a regra uma vez:

```bash
sudo ./scripts/install-viture-udev-rule.sh
```

Depois, desconecte e reconecte o óculos, ou faça logout/login.

## Observação

O `viture_sdk/` está vendorizado no repositório e o diretório `sample/` do pacote original não faz parte do projeto.
A superfície central agora é gerada proceduralmente como workstation, sem depender de uma imagem mock.
