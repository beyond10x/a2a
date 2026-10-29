//! Acceptance 2: each client method against `a2a-server` with the echo agent on port 0, once
//! without and once with a tenant, asserting the value it answers.

mod common;

use a2a::discovery::GetExtendedAgentCard;
use a2a::tasks::{
    CancelTask, CreateTaskPushNotificationConfig, DeleteTaskPushNotificationConfig, GetTask,
    GetTaskPushNotificationConfig, ListTaskPushNotificationConfigs,
};
use a2a_client::{Error, ListTasksRequest};
use common::{card, echo, message, send, start_with, state, task_of};

const TENANTS: [Option<&str>; 2] = [None, Some("acme")];

#[test]
fn agent_card_answers_the_built_in_card() {
    let server = echo();
    let card = server.client().agent_card().expect("the public card");
    assert_eq!(card.name, a2a_server::default_card().name);
    assert_eq!(card, a2a_server::default_card());
}

#[test]
fn extended_agent_card_answers_the_extended_card_for_the_bearer_token() {
    let mut settings = card(|card| card.capabilities.extended_agent_card = Some(true));
    let mut extended = settings.card.clone();
    extended.name = "the extended card".to_owned();
    settings.extended_card = Some(extended);
    let server = start_with(
        settings,
        a2a_server::Limits::DEFAULT,
        a2a_server::Access {
            extended_card_token: a2a_server::BearerToken::new("t0ken"),
        },
    );
    let client = server.client().with_bearer("t0ken");
    for tenant in TENANTS {
        let card = client
            .extended_agent_card(&GetExtendedAgentCard {
                tenant: tenant.map(str::to_owned),
            })
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"));
        assert_eq!(card.name, "the extended card", "{tenant:?}");
    }
}

#[test]
fn send_message_answers_a_new_task() {
    let server = echo();
    let client = server.client();
    for tenant in TENANTS {
        let first = task_of(&client, &send(message("hello"), tenant));
        let second = task_of(&client, &send(message("hello"), tenant));
        assert!(!first.id.0.is_empty(), "{tenant:?}");
        assert_ne!(first.id, second.id, "{tenant:?}: a task id was reused");
        assert_eq!(state(&first), "TASK_STATE_COMPLETED", "{tenant:?}");
        let echoed = first
            .artifacts
            .as_ref()
            .and_then(|artifacts| artifacts.first())
            .and_then(|artifact| artifact.parts.first())
            .and_then(|part| part.text.clone());
        assert_eq!(echoed.as_deref(), Some("hello"), "{tenant:?}");
    }
}

#[test]
fn get_task_answers_the_task_sent() {
    let server = echo();
    let client = server.client();
    for tenant in TENANTS {
        let sent = task_of(&client, &send(message("hello"), tenant));
        let read = client
            .get_task(&GetTask {
                tenant: tenant.map(str::to_owned),
                id: sent.id.clone(),
                history_length: None,
            })
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"));
        assert_eq!(read.id, sent.id, "{tenant:?}");
        assert_eq!(read.status.state, sent.status.state, "{tenant:?}");
    }
}

#[test]
fn list_tasks_lists_the_task_sent() {
    let server = echo();
    let client = server.client();
    for tenant in TENANTS {
        let sent = task_of(&client, &send(message("hello"), tenant));
        let listed = client
            .list_tasks(&ListTasksRequest {
                tenant: tenant.map(str::to_owned),
                ..ListTasksRequest::default()
            })
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"));
        assert!(
            listed.tasks.iter().any(|task| task.id == sent.id),
            "{tenant:?}: {listed:?}"
        );
        assert_eq!(listed.total_size, 1, "{tenant:?}: one task per tenant");
    }
}

#[test]
fn cancel_task_answers_the_task_canceled() {
    let server = echo();
    let client = server.client();
    for tenant in TENANTS {
        let waiting = task_of(&client, &send(message(""), tenant));
        assert_eq!(state(&waiting), "TASK_STATE_INPUT_REQUIRED", "{tenant:?}");
        let canceled = client
            .cancel_task(&CancelTask {
                tenant: tenant.map(str::to_owned),
                id: waiting.id.clone(),
                metadata: None,
            })
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"));
        assert_eq!(canceled.id, waiting.id, "{tenant:?}");
        assert_eq!(state(&canceled), "TASK_STATE_CANCELED", "{tenant:?}");
    }
}

#[test]
fn push_notification_configs_are_created_listed_read_and_deleted() {
    let server = echo();
    let client = server.client();
    for tenant in TENANTS {
        let owned = tenant.map(str::to_owned);
        let task = task_of(&client, &send(message(""), tenant)).id;
        let created = client
            .create_task_push_notification_config(&CreateTaskPushNotificationConfig {
                tenant: owned.clone(),
                id: None,
                task_id: task.clone(),
                url: "https://example.invalid/hook".to_owned(),
                token: Some("hook-token".to_owned()),
                authentication: None,
            })
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"));
        assert_eq!(created.url, "https://example.invalid/hook", "{tenant:?}");
        assert_eq!(created.task_id.as_ref(), Some(&task), "{tenant:?}");
        let id = created.id.clone().expect("the server names the config");

        let listed = client
            .list_task_push_notification_configs(&ListTaskPushNotificationConfigs {
                tenant: owned.clone(),
                task_id: task.clone(),
                page_size: None,
                page_token: None,
            })
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"));
        assert_eq!(
            listed.configs.as_deref(),
            Some(&[created.clone()][..]),
            "{tenant:?}"
        );

        let get = GetTaskPushNotificationConfig {
            tenant: owned.clone(),
            task_id: task.clone(),
            id: id.clone(),
        };
        let read = client
            .get_task_push_notification_config(&get)
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"));
        assert_eq!(read, created, "{tenant:?}");

        client
            .delete_task_push_notification_config(&DeleteTaskPushNotificationConfig {
                tenant: owned,
                task_id: task,
                id,
            })
            .unwrap_or_else(|error| panic!("{tenant:?}: {error}"));
        match client.get_task_push_notification_config(&get) {
            Err(Error::Status { http, status, .. }) => {
                assert_eq!(http, 404, "{tenant:?}");
                assert_eq!(status, "NOT_FOUND", "{tenant:?}");
            }
            other => panic!("{tenant:?}: a deleted config was answered {other:?}"),
        }
    }
}
