use esp_idf_svc::hal::{
    gpio::{Gpio5, Gpio6},
    prelude::Peripherals,
    uart::{config, UartDriver},
    units::Hertz,
};

use std::{thread, time::Duration};

const AT: &str = r#"AT"#;
const QMT_VERSION: &str = r#"AT+QMTCFG="version",0,3"#;
const QMT_SSL: &str = r#"AT+QMTCFG="SSL",0,1,0"#;
const QMT_RECV_MODE: &str = r#"AT+QMTCFG="recv/mode",0,0,1"#;
const SSL_VERSION: &str = r#"AT+QSSLCFG="sslversion",0,4"#;
const SSL_CIPHERSUITE: &str = r#"AT+QSSLCFG="ciphersuite",0,0XFFFF"#;
const SSL_SECLEVEL: &str = r#"AT+QSSLCFG="seclevel",0,0"#;
const SSL_CACERT: &str = r#"AT+QSSLCFG="cacert",0,"changeMECertName""#;
const SSL_IGNORE_INVALID_CERT_SIGN: &str = r#"AT+QSSLCFG="ignoreinvalidcertsign",0,1"#;
const SSL_SNI: &str = r#"AT+QSSLCFG="sni",0,1"#;
const QMTOPEN_COMMAND: &str = r#"AT+QMTOPEN=0,"changme.url.com",8883"#;
const QMTSUBSTART_COMMAND: &str = r#"AT+QMTSUB=0,1,"SUB/start",0"#;
const QMTSUBEND_COMMAND: &str = r#"AT+QMTSUB=0,1,"SUB/end",0"#;
const QMTSUBSTATUS_COMMAND: &str = r#"AT+QMTSUB=0,1,"SUB/status",0"#;
#[rustfmt::skip]
const QMTCONN_COMMAND: &str = r#"AT+QMTCONN=0,"U2","usernameChangeMe","passwordChangeMe""#;

const MQTT_CONNECTION_COMMAND_SEQ: [&str; 15] = [
    AT,
    QMT_VERSION,
    QMT_SSL,
    QMT_RECV_MODE,
    SSL_VERSION,
    SSL_CIPHERSUITE,
    SSL_SECLEVEL,
    SSL_CACERT,
    SSL_IGNORE_INVALID_CERT_SIGN,
    SSL_SNI,
    QMTOPEN_COMMAND,
    QMTCONN_COMMAND,
    QMTSUBSTART_COMMAND,
    QMTSUBEND_COMMAND,
    QMTSUBSTATUS_COMMAND,
];

fn main() {
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_svc::sys::link_patches();

    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    let at_tx_pin = peripherals.pins.gpio5;
    let at_rx_pin = peripherals.pins.gpio6;

    let at_config = config::Config::new().baudrate(Hertz(115200));
    let at_uart = Some(
        UartDriver::new(
            peripherals.uart1,
            at_tx_pin,
            at_rx_pin,
            Option::<Gpio5>::None,
            Option::<Gpio6>::None,
            &at_config,
        )
        .unwrap(),
    );

    let at_uart = at_uart.unwrap();
    log::info!("Hello, world!");

    for i in 0..=14 {
        let command_str = MQTT_CONNECTION_COMMAND_SEQ[i];
        let command = format!("{}\r\n", command_str);
        log::info!("Sending command: {:?}", command_str);
        at_uart.write(command.as_bytes()).unwrap();
        if command_str == QMTOPEN_COMMAND {
            log::info!("Waiting for 5 seconds before sending the next command...");
            thread::sleep(Duration::from_millis(800));
        } else {
            log::info!("Waiting for 1 second before sending the next command...");
            thread::sleep(Duration::from_millis(500));
        }
        let mut buf = [0; 1024];
        let len = at_uart.read(&mut buf, 1000).unwrap();
        let response = String::from_utf8_lossy(&buf[..len]);
        log::info!("Response: {:?}", response);
    }
    loop {
        let mut buf = [0; 1024];
        if let Ok(len) = at_uart.read(&mut buf, 1000) {
            let response = String::from_utf8_lossy(&buf[..len]);
            log::info!("Response: {:?}", response);
            if response.contains("OK") {
                break;
            }
        } else if let Err(e) = at_uart.read(&mut buf, 1000) {
            log::error!("Failed to read from UART: {}", e);
        }
    }
}
