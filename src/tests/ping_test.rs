#[cfg(test)]
mod ping_test{
    use crate::control_packet;

    //use super::*;

    #[test]
    fn handle_validping_returnsbytearray()
    {
        //arrange
        let mut package: [u8; 8192] = [0; 8192];
        package[0] = 192;
        package[1] = 0;
        let length:usize = 2;
        //act
        let result: Result<[u8; 2], &str> = control_packet::ping::handle(package, length);
        //assert
        assert_eq!(result, Ok([208, 0]));
        
    }

    #[test]
    fn handle_reservedbit3set_returnserr()
    {
        //arrange
        let mut package: [u8; 8192] = [0; 8192];
        package[0] = 200;
        package[1] = 0;
        let length:usize = 2;
        //act
        let result: Result<[u8; 2], &str> = control_packet::ping::handle(package, length);
        //assert
        assert_eq!(result, Err("Reserved bits MUST not be set"));
        
    }

    #[test]
    fn handle_reservedbit2set_returnserr()
    {
        //arrange
        let mut package: [u8; 8192] = [0; 8192];
        package[0] = 196;
        package[1] = 0;
        let length:usize = 2;
        //act
        let result: Result<[u8; 2], &str> = control_packet::ping::handle(package, length);
        //assert
        assert_eq!(result, Err("Reserved bits MUST not be set"));
        
    }

    #[test]
    fn handle_reservedbit1set_returnserr()
    {
        //arrange
        let mut package: [u8; 8192] = [0; 8192];
        package[0] = 194;
        package[1] = 0;
        let length:usize = 2;
        //act
        let result: Result<[u8; 2], &str> = control_packet::ping::handle(package, length);
        //assert
        assert_eq!(result, Err("Reserved bits MUST not be set"));
        
    }

    #[test]
    fn handle_reservedbit0set_returnserr()
    {
        //arrange
        let mut package: [u8; 8192] = [0; 8192];
        package[0] = 193;
        package[1] = 0;
        let length:usize = 2;
        //act
        let result = control_packet::ping::handle(package, length);
        //assert
        assert_eq!(result, Err("Reserved bits MUST not be set"));
        
    }

    #[test]
    fn handle_bytearray3long_returnserr()
    {
        //arrange
        let mut package: [u8; 8192] = [0; 8192];
        package[0] = 192;
        package[1] = 1;
        package[2] = 1;
        let length:usize = 2;
        //act
        let result = control_packet::ping::handle(package, length);
        //assert
        assert_eq!(result, Err("No payload expected here"));
        
    }

    #[test]
    fn handle_lengthis3_returnserr()
    {
        //arrange
        let mut package: [u8; 8192] = [0; 8192];
        package[0] = 192;
        package[1] = 0;
        let length:usize = 3;
        //act
        let result = control_packet::ping::handle(package, length);
        //assert
        assert_eq!(result, Err("No payload expected here"));
        
    }
}