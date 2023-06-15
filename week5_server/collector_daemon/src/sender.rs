use shared_data::{
    decode_response_v1, CollectorCommandV1, CollectorResponseV1, DATA_COLLECTOR_ADDRESS,
};
use std::{
    collections::VecDeque,
    io::{Read, Write},
};

use crate::CollectorError;

pub fn send_command(command: CollectorCommandV1) -> Result<(), CollectorError> {
    let mut stream = std::net::TcpStream::connect(DATA_COLLECTOR_ADDRESS)
        .map_err(|_| CollectorError::UnableToConnect)?;

    // Loop over receive
    let bytes = shared_data::encode_v1(command);
    println!("Encoded {} bytes", bytes.len());

    stream
        .write_all(&bytes)
        .map_err(|_| CollectorError::UnableToConnect)?;
    // End Loop

    Ok(())
}

pub fn send_queue(queue: &mut VecDeque<Vec<u8>>, collector_id: u128) -> Result<(), CollectorError> {
    // Connect
    let mut stream = std::net::TcpStream::connect(DATA_COLLECTOR_ADDRESS)
        .map_err(|_| CollectorError::UnableToConnect)?;

    // Send every queue item
    let mut buf = vec![0u8; 512];
    while let Some(command) = queue.pop_front() {
        if stream.write_all(&command).is_err() {
            queue.push_front(command);
            return Err(CollectorError::UnableToConnect);
        }
        let bytes_read = stream
            .read(&mut buf)
            .map_err(|_| CollectorError::UnableToConnect)?;
        if bytes_read == 0 {
            queue.push_front(command);
            return Err(CollectorError::UnableToConnect);
        }
        let ack = decode_response_v1(&buf[0..bytes_read]);
        if ack != CollectorResponseV1::Ack {
            queue.push_front(command);
            return Err(CollectorError::UnableToConnect);
        } else {
            // Comment this out for production
            println!("Ack received");
        }
    }

    // Ask for work
    let bytes = shared_v3::encode_v1(&shared_v3::CollectorCommandV1::RequestWork(collector_id));
    if stream.write_all(&bytes).is_err() {
        return Err(CollectorError::UnableToSendData);
    }
    let bytes_read = stream
        .read(&mut buf)
        .map_err(|_| CollectorError::UnableToReceiveData)?;
    if bytes_read == 0 {
        return Err(CollectorError::UnableToReceiveData);
    }
    let work = decode_response_v1(&buf[0..bytes_read]);
    match work {
        CollectorResponseV1::NoWork => {}
        CollectorResponseV1::Task(task) => {
            println!("Task received: {task:?}");
        }
        _ => {}
    }

    Ok(())
}
