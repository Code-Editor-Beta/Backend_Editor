// /**
//  * socket to sync files to users
//  */
// async fn ws_handler_file(
//     ws: WebSocketUpgrade,
//     Path((project_id, filename)): Path<(String, String)>,
//     State(state): State<crate::AppState>,
// ) -> impl IntoResponse {
//     //get bcast from AppState
//     let key = format!("{}/{}", project_id, filename);
//     if let Some(room) = state.rooms.get(&key) {
//         let bcast = Arc::clone(&room);
//         println!("Connecting to socket");
//         ws.on_upgrade(move |socket| file_peer(socket, bcast.clone()))
//     } else {
//         (
//             StatusCode::NOT_FOUND,
//             format!("Room with filename '{}' not found", filename),
//         )
//             .into_response()
//     };
// }

// pub async fn file_peer(ws: WebSocket, room: Arc<BroadcastGroup>) {
//     let (sink, stream) = ws.split();
//     let sink = Arc::new(Mutex::new(AxumSink::from(sink)));
//     let stream = AxumStream::from(stream);
//     info!("Sharing snapshot of folders");
//     let snapshot = {
//         let awareness = room.awareness().read().await;
//         let doc = awareness.doc();
//         let txn = doc.transact();

//         txn.encode_state_as_update_v1(&StateVector::default())
//     };
//     if sink.lock().await.send(snapshot).await.is_err() {
//         return; // client disconnected before snapshot delivered
//     }
//     info!("Folder Snapshot Sent");
//     let sub = room.subscribe(sink.clone(), stream);
//     match sub.completed().await {
//         Ok(_) => println!("broadcasting for channel finished successfully"),
//         Err(e) => eprintln!("broadcasting for channel finished abruptly: {}", e),
//     }
// }

// /**
//  * function to open file
// //  */
// #[axum::debug_handler]
// pub async fn open_file(
//     Path(OpenFileParams {
//         project_id,
//         filename,
//     }): Path<OpenFileParams>,
//     State(state): State<AppState>,
// ) -> impl IntoResponse {
//     match services::crdt_ops::handle_open_file(&state, &project_id, &filename).await {
//         Ok(_) => StatusCode::OK,
//         Err(e) => {
//             eprintln!("Failed to open file");
//             StatusCode::INTERNAL_SERVER_ERROR
//         }
//     }
// }
