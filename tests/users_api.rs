use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use tower::ServiceExt;

use project_manager_api::{routes::create_router, AppState};


#[tokio::test]
async fn update_user_status_without_token_returns_401() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let request = Request::builder()
        .method("PATCH")
        .uri("/users/1/status")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"status": false}"#))
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}


#[tokio::test]
async fn update_user_status_with_invalid_token_returns_401() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let request = Request::builder()
        .method("PATCH")
        .uri("/users/1/status")
        .header("content-type", "application/json")
        .header("Authorization", "Bearer invalid-token")
        .body(Body::from(r#"{"status": false}"#))
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}


#[tokio::test]
async fn update_user_status_with_member_token_returns_403() {
    use project_manager_api::token::create_access_token;

    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let secret = "test-secret";

    let state = AppState {
        pool,
        jwt_secret: secret.to_string(),
    };

    let app = create_router(state);

    let token = create_access_token(1, "member", secret)
        .expect("failed to create access token");

    let request = Request::builder()
        .method("PATCH")
        .uri("/users/2/status")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::from(r#"{"status": false}"#))
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}


#[tokio::test]
async fn admin_cannot_deactivate_own_account() {
    use project_manager_api::token::create_access_token;

    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let secret = "test-secret";

    let state = AppState {
        pool,
        jwt_secret: secret.to_string(),
    };

    let app = create_router(state);

    // The token belongs to user 1 with the admin role.
    let token = create_access_token(1, "admin", secret)
        .expect("failed to create access token");

    let request = Request::builder()
        .method("PATCH")
        .uri("/users/1/status")
        .header("content-type", "application/json")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::from(r#"{"status": false}"#))
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}


#[tokio::test]
async fn update_user_role_without_token_returns_401() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let request = Request::builder()
        .method("PATCH")
        .uri("/users/1/role")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"role": "manager"}"#))
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn update_user_role_with_invalid_token_returns_401() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let request = Request::builder()
        .method("PATCH")
        .uri("/users/1/role")
        .header("Authorization", "Bearer invalid-token")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"role": "manager"}"#))
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn update_user_role_with_member_token_returns_403() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let token = project_manager_api::token::create_access_token(
        1,
        "member",
        "test-secret",
    )
    .expect("failed to create access token");

    let request = Request::builder()
        .method("PATCH")
        .uri("/users/2/role")
        .header("Authorization", format!("Bearer {}", token))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"role": "manager"}"#))
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn admin_cannot_change_own_role() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let token = project_manager_api::token::create_access_token(
        1,
        "admin",
        "test-secret",
    )
    .expect("failed to create access token");

    let request = Request::builder()
        .method("PATCH")
        .uri("/users/1/role")
        .header("Authorization", format!("Bearer {}", token))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"role": "manager"}"#))
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn update_user_role_with_invalid_role_returns_400() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let token = project_manager_api::token::create_access_token(
        1,
        "admin",
        "test-secret",
    )
    .expect("failed to create access token");

    let request = Request::builder()
        .method("PATCH")
        .uri("/users/2/role")
        .header("Authorization", format!("Bearer {}", token))
        .header("content-type", "application/json")
        .body(Body::from(r#"{"role": "supervisor"}"#))
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn get_users_without_token_returns_401() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let request = Request::builder()
        .method("GET")
        .uri("/users")
        .body(Body::empty())
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_users_with_member_token_returns_403() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let token = project_manager_api::token::create_access_token(
        1,
        "member",
        "test-secret",
    )
    .expect("failed to create access token");

    let request = Request::builder()
        .method("GET")
        .uri("/users")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}


#[tokio::test]
async fn get_user_by_id_without_token_returns_401() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let request = Request::builder()
        .method("GET")
        .uri("/users/1")
        .body(Body::empty())
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}


#[tokio::test]
async fn get_another_user_with_member_token_returns_403() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let token = project_manager_api::token::create_access_token(
        1,
        "member",
        "test-secret",
    )
    .expect("failed to create access token");

    let request = Request::builder()
        .method("GET")
        .uri("/users/2")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::FORBIDDEN);
}


#[tokio::test]
async fn get_user_by_id_with_invalid_token_returns_401() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let request = Request::builder()
        .method("GET")
        .uri("/users/1")
        .header("Authorization", "Bearer invalid-token")
        .body(Body::empty())
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}



#[tokio::test]
async fn update_user_status_with_malformed_token_returns_401() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let request = Request::builder()
        .method("PATCH")
        .uri("/users/1/status")
        .header("Authorization", "Bearer invalid-token")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"status": true}"#))
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}


#[tokio::test]
async fn get_current_user_without_token_returns_401() {
    let pool = sqlx::SqlitePool::connect("sqlite::memory:")
        .await
        .expect("failed to connect to in-memory database");

    let state = AppState {
        pool,
        jwt_secret: "test-secret".to_string(),
    };

    let app = create_router(state);

    let request = Request::builder()
        .method("GET")
        .uri("/auth/me")
        .body(Body::empty())
        .unwrap();

    let response = app
        .oneshot(request)
        .await
        .expect("request failed");

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}





