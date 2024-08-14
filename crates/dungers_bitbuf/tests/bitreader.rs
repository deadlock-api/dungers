use dungers_bitbuf::BitReader;

#[test]
fn test_read_ubit64_overflow() {
    let buf = [0xffu8; 8];
    let mut r = BitReader::new(&buf);

    assert!(r.read_ubit64(u64::BITS as usize).is_ok());
    assert!(r.read_ubit64(1).is_err());
}

#[test]
fn test_read_ubit64_multiple_reads() {
    let mut buf = [0u8; 8];
    buf[0] = 0b1100_101;
    let mut r = BitReader::new(&buf);

    assert_eq!(r.read_ubit64(3).unwrap(), 0b101);
    assert_eq!(r.read_ubit64(4).unwrap(), 0b1100);
}

#[test]
fn test_read_ubit64_spanning_blocks() {
    let mut buf = [0xff; 16];
    buf[8] = 0xaa;
    let mut r = BitReader::new(&buf);

    r.read_ubit64(60).unwrap();

    // read 8 bits that span across the first and second block
    let result = r.read_ubit64(8).unwrap();
    // the result should be 4 bits from the end of the first block and 4 bits from the
    // start of the second block
    assert_eq!(result, 0xaf);
}
