#[cfg(feature = "edr")]
#[tokio::test]
async fn edr() -> anyhow::Result<()> {
    use std::env;
    use std::net::{SocketAddr, TcpListener};

    use axum::http::Request;
    use geojson::{Geometry, Value};
    use url::Url;
    use uuid::Uuid;

    use ogcapi::import::{self, Args};
    use ogcapi_drivers::postgres::Db;
    use ogcapi_types::{common::Crs, edr::Query, features::FeatureCollection};

    // setup app
    dotenv::dotenv().ok();

    // tracing_subscriber::fmt::init();

    let mut database_url = Url::parse(&env::var("DATABASE_URL")?)?;
    let daatbase_name = Uuid::new_v4().to_string();
    database_url.set_path(&daatbase_name);

    let db = Db::setup_with(&database_url, &daatbase_name, true)
        .await
        .expect("Setup database");

    let state = ogcapi_services::State::new(db, ogcapi_services::OPENAPI);

    let app = ogcapi_services::app(state).await;

    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .expect("")
            .serve(app.into_make_service())
            .await
            .unwrap();
    });

    let client = hyper::Client::new();

    // load data
    let args = Args::new(
        "../data/ne_110m_admin_0_countries.geojson",
        "countries",
        &database_url,
    );
    import::geojson::load(args, false).await?;

    let args = Args::new(
        "../data/ne_110m_populated_places.geojson",
        "places",
        &database_url,
    );
    import::geojson::load(args, false).await?;

    // import::geojson::load(
    //     Args {
    //         input: PathBuf::from_str("../data/ne_10m_railroads_north_america.geojson")?,
    //         collection: "railroads".to_string(),
    //         ..Default::default()
    //     },
    //     &database_url,
    //     false,
    // )
    // .await?;

    // query position
    let query = Query {
        coords: "POINT(2600000 1200000)".to_string(),
        parameter_name: Some("NAME,ISO_A2,CONTINENT".to_string()),
        crs: Crs::from(2056),
        ..Default::default()
    };

    let res = client
        .request(
            Request::builder()
                .method(axum::http::Method::GET)
                .uri(format!(
                    "http://{}/collections/countries/position?{}",
                    addr,
                    serde_qs::to_string(&query)?
                ))
                .body(hyper::Body::empty())?,
        )
        .await?;

    let (parts, body) = res.into_parts();

    assert_eq!(200, parts.status);

    let body = hyper::body::to_bytes(body).await?;
    let fc: FeatureCollection = serde_json::from_slice(&body)?;

    assert_eq!(fc.number_matched, Some(1));
    assert_eq!(fc.number_returned, Some(1));
    let feature = &fc.features[0];
    assert_eq!(feature.properties.as_ref().unwrap().len(), 3);
    assert_eq!(
        feature.properties.as_ref().unwrap()["NAME"].as_str(),
        Some("Switzerland")
    );

    // query area
    let query = Query {
        coords: "POLYGON((6 45, 6 49, 9 49, 9 45, 6 45))".to_string(),
        parameter_name: Some("NAME,ISO_A2,ADM0NAME".to_string()),
        ..Default::default()
    };

    let res = client
        .request(
            Request::builder()
                .method(axum::http::Method::GET)
                .uri(format!(
                    "http://{}/collections/places/area?{}",
                    addr,
                    serde_qs::to_string(&query)?
                ))
                .body(hyper::Body::empty())?,
        )
        .await?;

    assert_eq!(200, res.status());

    let body = hyper::body::to_bytes(res.into_body()).await?;
    let fc: FeatureCollection = serde_json::from_slice(&body)?;

    assert_eq!(fc.number_matched, Some(2));
    assert_eq!(fc.number_returned, Some(2));
    let feature = &fc
        .features
        .into_iter()
        .find(|f| f.properties.as_ref().unwrap()["NAME"].as_str() == Some("Bern"));
    assert!(feature.is_some());

    // query radius
    let query = Query {
        coords: "POINT(7.5 47)".to_string(),
        parameter_name: Some("NAME,ISO_A2,ADM0NAME".to_string()),
        within: Some("1000".to_string()),
        within_units: Some("km".to_string()),
        ..Default::default()
    };

    let res = client
        .request(
            Request::builder()
                .method(axum::http::Method::GET)
                .uri(format!(
                    "http://{}/collections/countries/radius?{}",
                    addr,
                    serde_qs::to_string(&query)?
                ))
                .body(hyper::Body::empty())?,
        )
        .await?;

    assert_eq!(200, res.status());

    let body = hyper::body::to_bytes(res.into_body()).await?;
    let mut fc: FeatureCollection = serde_json::from_slice(&body)?;

    for mut feature in fc.features.iter_mut() {
        feature.geometry = Geometry::new(Value::Point(vec![0.0, 0.0]));
    }

    tracing::debug!("{}", serde_json::to_string_pretty(&fc.number_matched)?);
    // assert_eq!(features.number_matched, Some(19));
    // assert_eq!(features.number_returned, Some(19));
    // let feature = &features
    //     .features
    //     .into_iter()
    //     .find(|f| f.properties.as_ref().unwrap().0["NAME"].as_str() == Some("Bern"));
    // assert!(feature.is_some());

    Ok(())
}
