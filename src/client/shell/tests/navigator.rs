// ABOUTME: Go to picker tests for search at open, tab rows, workspace jumps, and row selection.
use super::copy::navigator_scale_snapshot;
use super::*;

fn navigator_state(snapshot: ClientShellSnapshot) -> ClientShellState {
    let mut state = ClientShellState::new(ClientShellConfig::from_config(&Config::default()));
    state.set_snapshot(Box::new(snapshot));
    let mut pane_surface = surface();
    pane_surface.panes[0].pane_id = "pane_0_0_0".into();
    state.set_pane_surface(pane_surface);
    state.open_navigator_overlay();
    state
}

fn press(state: &mut ClientShellState, code: KeyCode) -> ClientShellInput {
    state.handle_raw_events(vec![RawInputEvent::Key(crate::input::TerminalKey::new(
        code,
        KeyModifiers::empty(),
    ))])
}

fn navigator(state: &ClientShellState) -> &ClientNavigatorOverlay {
    let Some(ClientShellOverlay::Navigator(navigator)) = &state.overlay else {
        panic!("navigator");
    };
    navigator
}

fn rows(state: &ClientShellState) -> Vec<ClientNavigatorRow> {
    render::client_navigator_rows(
        &state.endpoints,
        &state.active_endpoint_id,
        navigator(state),
    )
}

fn pane(id: &str) -> ClientNavigatorTarget {
    ClientNavigatorTarget::Pane {
        endpoint_id: ClientEndpointId::Local,
        pane_id: id.into(),
    }
}

fn tab(id: &str) -> ClientNavigatorTarget {
    ClientNavigatorTarget::Tab {
        endpoint_id: ClientEndpointId::Local,
        tab_id: id.into(),
    }
}

fn workspace(id: &str) -> ClientNavigatorTarget {
    ClientNavigatorTarget::Workspace {
        endpoint_id: ClientEndpointId::Local,
        workspace_id: id.into(),
    }
}

fn select(state: &mut ClientShellState, target: ClientNavigatorTarget) {
    let Some(ClientShellOverlay::Navigator(navigator)) = state.overlay.as_mut() else {
        panic!("navigator");
    };
    navigator.selected = Some(target);
    navigator.selected_row = 0;
}

fn selected(state: &ClientShellState) -> Option<ClientNavigatorTarget> {
    navigator(state).selected.clone()
}

#[test]
fn navigator_opens_with_search_active() {
    let mut state = navigator_state(navigator_scale_snapshot(2, 1, 2));
    assert!(navigator(&state).search_focused);

    assert!(state.handle_input_bytes(b"term").actions.is_empty());
    assert_eq!(navigator(&state).query.as_str(), "term");

    press(&mut state, KeyCode::Esc);
    assert!(!navigator(&state).search_focused);
    assert_eq!(navigator(&state).query.as_str(), "term");
    let before = selected(&state);
    press(&mut state, KeyCode::Char('j'));
    assert_eq!(navigator(&state).query.as_str(), "term");
    assert_ne!(selected(&state), before);

    press(&mut state, KeyCode::Esc);
    assert!(state.overlay.is_none());
}

#[test]
fn navigator_lists_tab_rows_for_multi_tab_workspace() {
    let state = navigator_state(navigator_scale_snapshot(2, 2, 2));
    let rows = rows(&state);
    let layout = rows
        .iter()
        .map(|row| (row.depth, row.target.clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        layout[..7],
        [
            (0, workspace("workspace_0")),
            (1, tab("tab_0_0")),
            (2, pane("pane_0_0_0")),
            (2, pane("pane_0_0_1")),
            (1, tab("tab_0_1")),
            (2, pane("pane_0_1_0")),
            (2, pane("pane_0_1_1")),
        ]
    );
    assert_eq!(rows[1].label, "tab 0");
    assert!(rows
        .iter()
        .filter(|row| matches!(row.target, ClientNavigatorTarget::Tab { .. }))
        .all(|row| !row.current && row.status.is_none()));
    assert_eq!(
        rows.iter().find(|row| row.current).map(|row| &row.target),
        Some(&pane("pane_0_0_0"))
    );

    let mut state = state;
    state.handle_input_bytes(b"pane_1_1_0");
    let filtered = self::rows(&state)
        .into_iter()
        .map(|row| row.target)
        .collect::<Vec<_>>();
    assert_eq!(
        filtered,
        [workspace("workspace_1"), tab("tab_1_1"), pane("pane_1_1_0")]
    );
}

#[test]
fn navigator_hides_tab_row_for_single_tab_workspace() {
    let state = navigator_state(navigator_scale_snapshot(2, 1, 2));
    let rows = rows(&state);
    assert!(rows
        .iter()
        .all(|row| !matches!(row.target, ClientNavigatorTarget::Tab { .. })));
    assert!(rows
        .iter()
        .filter(|row| matches!(row.target, ClientNavigatorTarget::Pane { .. }))
        .all(|row| row.depth == 1));
}

#[test]
fn navigator_enter_on_tab_row_focuses_tab() {
    let mut state = navigator_state(navigator_scale_snapshot(1, 2, 1));
    assert_eq!(selected(&state), Some(pane("pane_0_0_0")));
    press(&mut state, KeyCode::Down);
    assert_eq!(selected(&state), Some(tab("tab_0_1")));

    let accept = press(&mut state, KeyCode::Enter);
    let [ClientShellAction::Endpoint { request, .. }] = &accept.actions[..] else {
        panic!("tab row should focus through the endpoint API");
    };
    assert!(matches!(
        &request.method,
        crate::api::schema::Method::TabFocus(target) if target.tab_id == "tab_0_1"
    ));
    assert!(state.overlay.is_none());
}

#[test]
fn navigator_left_right_jumps_workspaces_with_tab_rows() {
    let mut snapshot = navigator_scale_snapshot(3, 2, 2);
    snapshot.tabs.retain(|tab| tab.tab_id != "tab_1_1");
    snapshot.panes.retain(|pane| pane.tab_id != "tab_1_1");
    let mut state = navigator_state(snapshot);
    press(&mut state, KeyCode::Esc);

    select(&mut state, pane("pane_0_1_1"));
    press(&mut state, KeyCode::Right);
    assert_eq!(selected(&state), Some(pane("pane_1_0_0")));
    press(&mut state, KeyCode::Right);
    assert_eq!(selected(&state), Some(pane("pane_2_0_0")));
    press(&mut state, KeyCode::Left);
    assert_eq!(selected(&state), Some(pane("pane_1_0_0")));
    press(&mut state, KeyCode::Left);
    assert_eq!(selected(&state), Some(pane("pane_0_0_0")));

    select(&mut state, tab("tab_2_1"));
    press(&mut state, KeyCode::Left);
    assert_eq!(selected(&state), Some(pane("pane_1_0_0")));
    select(&mut state, tab("tab_0_1"));
    press(&mut state, KeyCode::Right);
    assert_eq!(selected(&state), Some(pane("pane_1_0_0")));
}

#[test]
fn navigator_selection_unique_when_target_repeats() {
    use super::super::aggregate_navigation::{navigator_selected_index, select_navigator_row};
    let row = |target: ClientNavigatorTarget| ClientNavigatorRow {
        depth: 0,
        label: String::new(),
        meta: String::new(),
        detail: String::new(),
        agent: None,
        status: None,
        stale: false,
        current: false,
        target,
    };
    let rows = [
        row(pane("pane_a")),
        row(workspace("workspace_0")),
        row(pane("pane_a")),
        row(pane("pane_b")),
        row(pane("pane_a")),
    ];
    let mut state = navigator_state(navigator_scale_snapshot(1, 1, 1));
    let Some(ClientShellOverlay::Navigator(navigator)) = state.overlay.as_mut() else {
        panic!("navigator");
    };
    for index in 0..rows.len() {
        select_navigator_row(navigator, &rows, index);
        assert_eq!(navigator_selected_index(&rows, navigator), Some(index));
    }
}
