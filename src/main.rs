use std::ffi::CString;
use std::io::{Error, Seek, SeekFrom};
use std::ptr;
use std::slice;

use libc::{c_void, close, lseek, open, read, write, O_CREAT, O_DIRECT, O_RDWR, SEEK_SET};
use std::fs::OpenOptions;
use std::io::{self, Read, Write};
use std::os::unix::fs::OpenOptionsExt;

fn test1() {
    let block_size = 4096;
    let time = unsafe { libc::time(ptr::null_mut()) };
    let path = format!("testfile1_{}", time);

    // Open the file with O_DIRECT
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .custom_flags(libc::O_DIRECT)
        .open(&path)
        .unwrap();

    // Allocate aligned buffer
    let mut buf = vec![0u8; block_size];
    let buf_ptr = buf.as_mut_ptr();
    // if (buf_ptr as usize) % block_size != 0 {
    //     // Buffer is not aligned
    //     return Err(io::Error::new(
    //         io::ErrorKind::Other,
    //         "Buffer is not aligned",
    //     ));
    // }

    // Prepare data to write
    let data = b"Hello, O_DIRECT!";
    unsafe {
        ptr::copy_nonoverlapping(data.as_ptr(), buf_ptr, data.len());
    }

    // Write data
    file.write_all(&buf).unwrap(); // This may not work as expected

    // Seek to the beginning
    file.seek(io::SeekFrom::Start(0)).unwrap();

    // Read data
    let mut read_buf = vec![0u8; block_size];
    file.read_exact(&mut read_buf).unwrap(); // This may not work as expected

    println!("Read data 1: {:?}", &read_buf[..data.len()]);
}

use libc::{free, posix_memalign};
use std::os::unix::io::AsRawFd;

fn test2() {
    // Path to the file
    let time = unsafe { libc::time(ptr::null_mut()) };
    let path = format!("testfile2_{}", time);

    // Block size (usually 4096 bytes)
    let block_size = 4096;

    // Open the file with O_DIRECT using OpenOptions
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .custom_flags(libc::O_DIRECT)
        .open(&path)
        .unwrap();

    // Get the raw file descriptor
    let fd = file.as_raw_fd();

    // Allocate aligned buffer
    let mut buf: *mut u8 = ptr::null_mut();
    let ret = unsafe {
        posix_memalign(
            &mut buf as *mut *mut u8 as *mut *mut c_void,
            block_size,
            block_size,
        )
    };
    if ret != 0 {
        return Err(Error::last_os_error()).unwrap();
    }

    unsafe {
        // Prepare data to write
        let data = b"Hello, O_DIRECT!";
        ptr::copy_nonoverlapping(data.as_ptr(), buf, data.len());

        // Zero out the rest of the buffer to match block_size
        let remaining = block_size - data.len();
        ptr::write_bytes(buf.add(data.len()), 0, remaining);

        // Write data
        let ret = write(fd, buf as *const c_void, block_size);
        if ret < 0 {
            free(buf as *mut c_void);
            return Err(Error::last_os_error()).unwrap();
        }

        // Seek to the beginning of the file
        file.seek(SeekFrom::Start(0)).unwrap();

        // Clear the buffer before reading
        ptr::write_bytes(buf, 0, block_size);

        // Read data
        let ret = read(fd, buf as *mut c_void, block_size);
        if ret < 0 {
            free(buf as *mut c_void);
            return Err(Error::last_os_error()).unwrap();
        }

        // Convert the read data to a slice and print it
        let read_data = slice::from_raw_parts(buf as *const u8, ret as usize);
        println!("Read data 2: {:?}", &read_data[0..data.len()]);

        // Clean up
        free(buf as *mut c_void);
    }
}

fn test3() {
    unsafe {
        let time = libc::time(ptr::null_mut());
        let path = CString::new(format!("testfile3_{}", time)).unwrap();

        // Open the file with O_DIRECT
        let fd = open(path.as_ptr(), O_CREAT | O_RDWR | O_DIRECT, 0o644);
        if fd < 0 {
            return Err(Error::last_os_error()).unwrap();
        }

        // Block size (usually 4096 bytes)
        let block_size = 4096;

        // Allocate aligned buffer
        let buf = libc::memalign(block_size, block_size);
        if buf.is_null() {
            close(fd);
            return Err(Error::last_os_error()).unwrap();
        }

        // Prepare data to write
        let data = b"Hello, O_DIRECT!";
        ptr::copy_nonoverlapping(data.as_ptr(), buf as *mut u8, data.len());

        // Zero out the rest of the buffer to match block_size
        let remaining = block_size - data.len();
        ptr::write_bytes((buf as *mut u8).add(data.len()), 0, remaining);

        // Write data
        let ret = write(fd, buf, block_size);
        if ret < 0 {
            libc::free(buf);
            close(fd);
            return Err(Error::last_os_error()).unwrap();
        }

        // Seek to the beginning of the file
        if lseek(fd, 0, SEEK_SET) < 0 {
            libc::free(buf);
            close(fd);
            return Err(Error::last_os_error()).unwrap();
        }

        // Clear the buffer before reading
        ptr::write_bytes(buf, 0, block_size);

        // Read data
        let ret = read(fd, buf, block_size);
        if ret < 0 {
            libc::free(buf);
            close(fd);
            return Err(Error::last_os_error()).unwrap();
        }

        // Convert the read data to a slice and print it
        let read_data = slice::from_raw_parts(buf as *const u8, ret as usize);
        println!("Read data 3: {:?}", &read_data[0..data.len()]);

        // Clean up
        libc::free(buf);
        close(fd);
    }
}

fn main() {
    test1();
    test2();
    test3();
}
