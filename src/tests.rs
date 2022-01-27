#[cfg(test)]
mod tests {

    #[test]
    fn can_read_fin_bit() {
        let frame = Frame::new();
        let message: [u8;128] = [];
        frame.parse(message);
        assert_eq!(frame.fin, true);
    }

}