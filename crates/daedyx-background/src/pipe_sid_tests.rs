//! Pipe-Härtung in der Einzelinstanz (Entscheid T4-7): Der Name hängt an der SID des Benutzers, nicht an einer
//! Umgebungsvariablen; die erste Instanz legt die Pipe über `daedyx-sys` an.
use crate::ids::{AppIds, pipe_name};
use crate::instance::{Role, acquire};
use crate::start;

const APP: AppIds = AppIds {
    aumid: "Daedyx.Background.Test",
    run_value: "DaedyxBackgroundTest",
    pipe_prefix: "daedyx-background-test-instanz",
};

#[test]
fn der_pipe_name_enthaelt_die_sid_des_benutzers() {
    let sid = daedyx_sys::pipe::user_sid().unwrap();
    assert_eq!(
        start::pipe(&APP),
        format!(r"\\.\pipe\daedyx-background-test-instanz-{sid}")
    );
}

#[test]
fn der_pipe_name_laesst_nur_buchstaben_ziffern_und_bindestriche_durch() {
    assert_eq!(pipe_name("p", r"S-1-5-18\..\x y"), r"\\.\pipe\p-S-1-5-18xy");
    assert_eq!(pipe_name("p", "test-42"), r"\\.\pipe\p-test-42");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn die_erste_instanz_laeuft_ueber_die_gehaertete_pipe() {
    let name = format!(
        r"\\.\pipe\daedyx-background-test-sid-{}",
        std::process::id()
    );
    let Role::Primary(_listener) = acquire(&name).unwrap() else {
        panic!("erster Start muss die Pipe bekommen");
    };
    // Verbinden geht nur über die Besitzerprüfung aus `daedyx-sys`; Rechte und Gegenproben prüft `daedyx-sys` selbst.
    let client = daedyx_sys::pipe::open_client(&name).unwrap();
    assert_eq!(
        daedyx_sys::pipe::pipe_owner_sid(&client).unwrap(),
        daedyx_sys::pipe::user_sid().unwrap()
    );
}
