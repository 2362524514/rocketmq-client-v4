use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use crate::protocols::body::message_queue::MessageQueue;
use crate::protocols::mq_command::MqCommand;
use crate::protocols::response_code;

#[derive(Debug)]
pub struct GetConsumerStatusBody {
    pub message_queue_table: HashMap<MessageQueue, i32>,
}

impl GetConsumerStatusBody {

    pub fn new_from_queues(mq_list: &Vec<MessageQueue>) -> Self {
        let mut table = HashMap::new();
        for mq in mq_list {
            let key = mq.clone();
            let val = key.queueId;
            table.insert(key, val);
        }
        GetConsumerStatusBody::new(table)
    }
    pub fn new(message_queue_table: HashMap<MessageQueue, i32>) -> Self {
        GetConsumerStatusBody {
            message_queue_table
        }
    }

    /// 因为这个json它不是标准的，需要单独处理
    pub fn to_json(&self) -> String {
        let mut body = String::new();
        body.push_str("{");
        body.push_str("\"messageQueueTable\":{");
        for (k,v) in &self.message_queue_table {
            let json = serde_json::to_string(k).unwrap();
            body.push_str(&json);
            body.push_str(":");
            body.push_str(&v.to_string());
            body.push_str(",");
        }
        body.push_str("}");
        body.push_str("}");
        body
    }

    pub async fn send_request(self, broker_stream: &mut TcpStream) {

        let body = self.to_json();
        let bytes = Vec::from(body);
        let cmd = MqCommand::new_with_body(response_code::SUCCESS, vec![], vec![], bytes);
        let opaque = cmd.opaque;
        let req_data = cmd.to_bytes();
        let req = broker_stream.write_all(&req_data).await;
        if req.is_err() {
            panic!("send GetConsumerStatusBody failed:{:?}", req.err());
        }
        let ret = MqCommand::read_from_stream_with_opaque(broker_stream, opaque).await;
        if ret.req_code == response_code::SUCCESS {
            println!("GetConsumerStatusBody success,  opaque:{}, resp opaque:{}", opaque, ret.opaque);
        } else {
            panic!("GetConsumerStatusBody failed, opaque:{}, resp opaque:{} req_code:{}, remark:{:?}",
                   opaque,ret.opaque, ret.req_code, String::from_utf8(ret.r_body));
        }
    }
}



#[cfg(test)]
mod test {
    use std::collections::HashMap;
    use crate::protocols::body::get_consumer_status_body::GetConsumerStatusBody;
    use crate::protocols::body::message_queue::MessageQueue;
    use crate::rocketmq::protocols::body::get_consumer_status_body::GetConsumerStatusBody;
    use crate::rocketmq::protocols::body::message_queue::MessageQueue;
    #[test]
    fn test_map_to_json() {
        let queue1 = MessageQueue::new("test_topic".to_string(), "test_broker".to_string(), 0);
        let queue2 = MessageQueue::new("test_topic".to_string(), "test_broker".to_string(), 1);
        let mut table = HashMap::new();
        table.insert(queue1, 0);
        table.insert(queue2, 1);
        let body = GetConsumerStatusBody {
            message_queue_table: table,
        };

        println!("{}", body.to_json());
    }
}