/* Dump Saleae Logic output to stdout.
   Adapted from SaleaeDeviceSdk-1.1.14/source/ConsoleDemo.cpp */

#include <SaleaeDeviceApi.h>

#include <memory>
#include <iostream>
#include <string>

#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <unistd.h>

#include <sys/types.h>
#include <sys/stat.h>
#include <fcntl.h>

void __stdcall OnConnect( U64 device_id, GenericInterface* device_interface, void* user_data );
void __stdcall OnDisconnect( U64 device_id, void* user_data );
void __stdcall OnReadData( U64 device_id, U8* data, U32 data_length, void* user_data );
void __stdcall OnWriteData( U64 device_id, U8* data, U32 data_length, void* user_data );
void __stdcall OnError( U64 device_id, void* user_data );

LogicInterface* gDeviceInterface = NULL;

#define LOG(...) fprintf(stderr, __VA_ARGS__)

U64 gLogicId = 0;
U32 gSampleRateHz = 2000000;

#define CMD_READ  thread_cmd[0]
#define CMD_WRITE thread_cmd[1]
int thread_cmd[2];

#define CMD_NOP   0
#define CMD_ERROR 1

int log_fd = 1;


/* When we are used as an Erlang port, we need to exit when stdin
   closes.  Saleae library is threaded, so just use another thread. */
void *mon_stdin(void *context) {
    for(;;) {
        uint8_t buf[1024];
        int rv = read(0, buf, sizeof(buf));
        if (rv == 0) { exit(0); }
	/* Non-blocking I/O seems to be enabled, so keep polling. */
	sleep(1);
    }
}

int main( int argc, char *argv[] ) {

    pthread_t mon_stdin_thread;
    pthread_create(&mon_stdin_thread, NULL, mon_stdin, NULL);

    DevicesManagerInterface::RegisterOnConnect( &OnConnect );
    DevicesManagerInterface::RegisterOnDisconnect( &OnDisconnect );
    DevicesManagerInterface::BeginConnect();

    if (argc > 2) {
        LOG("usage: %s <samplerate>\n", argv[0]);
        exit(1);
    }
    if (argc == 2) {
        gSampleRateHz = atoi(argv[1]);
    }
    
    LOG("Samplerate %d\n", gSampleRateHz);
    LOG("Waiting..\n");

    // Use unix pipe for inter-thread communication.
    pipe(thread_cmd);
    int cmd;
    while(sizeof(int) == read(CMD_READ, &cmd, sizeof(cmd))) {
        switch(cmd) {
        default:
            LOG("Ignoring unknown command %d\n", cmd);
        case CMD_NOP:
            break;
        case CMD_ERROR:
            /* On error, we just restart. */
            if (!gDeviceInterface) {
                LOG("CMD_ERROR: gDeviceInterface == NULL\n");
            }
            else {
                LOG("CMD_ERROR: starting read\n");
                gDeviceInterface->ReadStart();
            }
            break;
        }
    }
}

void write_fd(int fd, U8* data, U32 data_length) {
    int i = 0;
    while (i < data_length) {
        int remaining = data_length - i;
        int rv = write(fd, data + i, remaining);
        if (rv > 0) {
            i += rv;
        }
        else {
            LOG("stdout write error %d\n", rv);
            exit(1);
        }
    }
}


void __stdcall OnReadData( U64 device_id, U8* data, U32 data_length, void* user_data ) {
    // LOG(".");
    write_fd(log_fd, data, data_length);

    // We own, so need to delete.
    DevicesManagerInterface::DeleteU8ArrayPtr( data );
}

void __stdcall OnWriteData( U64 device_id, U8* data, U32 data_length, void* user_data )
{
    /* Not used */
}

void __stdcall OnError( U64 device_id, void* user_data )
{
    LOG("ERROR\n");
    // Notify main thread
    int cmd = CMD_ERROR;
    write(CMD_WRITE, &cmd, sizeof(cmd));
}

void __stdcall OnDisconnect( U64 device_id, void* user_data ) {
    if( device_id == gLogicId ) {
        LOG("Disconnect %08x\n", device_id);
        gDeviceInterface = NULL;
    }
}


void __stdcall OnConnect( U64 device_id, GenericInterface* device_interface, void* user_data )
{
    if( dynamic_cast<LogicInterface*>( device_interface ) != NULL ) {
        LOG("Connect %08x\n", device_id);

        gDeviceInterface = (LogicInterface*)device_interface;
        gLogicId = device_id;
        
        gDeviceInterface->RegisterOnReadData( &OnReadData );
        gDeviceInterface->RegisterOnWriteData( &OnWriteData );
        gDeviceInterface->RegisterOnError( &OnError );
        
        gDeviceInterface->SetSampleRateHz( gSampleRateHz );

        // Start automatically
        gDeviceInterface->ReadStart();

    }
}

