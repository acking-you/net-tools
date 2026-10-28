use super::*;

/// A loopback PCP peer verifies real responses, unchanged-address renewal,
/// rejection and deletion. No host gateway or environment is consulted.
#[tokio::test]
async fn observations_cover_renewal_failure_and_unconfirmed_release() {
    let gateway = tokio::net::UdpSocket::bind((Ipv4Addr::LOCALHOST, 5351))
        .await
        .unwrap();
    let peer = tokio::spawn(async move {
        let mut request = [0_u8; 1100];
        for attempt in 0..3 {
            let (len, source) = gateway.recv_from(&mut request).await.unwrap();
            assert_eq!(len, 60);
            let mut response = [0_u8; 60];
            response[0] = 2;
            response[1] = 0x81;
            response[4..8].copy_from_slice(&60_u32.to_be_bytes());
            response[24..60].copy_from_slice(&request[24..60]);
            response[42..44].copy_from_slice(&40000_u16.to_be_bytes());
            response[44..60]
                .copy_from_slice(&Ipv4Addr::new(203, 0, 113, 7).to_ipv6_mapped().octets());
            if attempt == 2 {
                response[3] = 2;
            }
            gateway.send_to(&response, source).await.unwrap();
        }
        let (len, _) = gateway.recv_from(&mut request).await.unwrap();
        assert_eq!(len, 60);
        assert_eq!(&request[4..8], &[0; 4]);
    });
    let (_, rx) = mpsc::channel(8);
    let (mut service, mut address) = Service::new(Config::default(), rx, Default::default());
    let events = Arc::new(std::sync::Mutex::new(Vec::new()));
    let observed = events.clone();
    service.observer = Observer::new(move |event| observed.lock().unwrap().push(event));
    for attempt in 0..3 {
        let result = mapping::Mapping::new_pcp(
            Protocol::Udp,
            Ipv4Addr::LOCALHOST,
            NonZeroU16::new(12345).unwrap(),
            Ipv4Addr::LOCALHOST,
            None,
        )
        .await;
        service.on_mapping_result(Ok(result));
        if attempt == 0 {
            address.changed().await.unwrap();
            address.borrow_and_update();
        } else {
            assert!(
                !address.has_changed().unwrap(),
                "same-address renewal and failure leave the lease active"
            );
        }
    }
    service.invalidate_mapping().await;
    tokio::time::timeout(Duration::from_secs(2), peer)
        .await
        .unwrap()
        .unwrap();
    let events = events.lock().unwrap();
    assert!(matches!(
        events.as_slice(),
        [
            Event::Active {
                protocol: MappingProtocol::Pcp,
                ..
            },
            Event::Active {
                protocol: MappingProtocol::Pcp,
                ..
            },
            Event::MappingFailed(_),
            Event::Releasing(_),
            Event::Released {
                confirmed: false,
                ..
            },
        ]
    ));
}
