# Sensor Core

Sensor Core is a core library for the Sensor Bridge and Sensor Display projects. These projects allow you to display
sensor information from one device on another device's screen. This document provides an overview of the Sensor Core
library.

## Sensor Bridge

Sensor Bridge is a cross-platform desktop application that collects sensor data. It's designed to reduce memory and CPU
consumption on the device collecting the data, as rendering is offloaded to the device running Sensor Display.

## Sensor Display

Sensor Display is part of a two-application system that receives sensor information from the Sensor Bridge application
and displays it on the screen. It's designed to reduce memory and CPU consumption on the device collecting the data, as
rendering is offloaded to the device running Sensor Display.

## Architecture

Both Sensor Bridge and Sensor Display are designed to work together. Sensor Bridge runs on the device collecting the
sensor data, and Sensor Display runs on a separate device with a connected display. The sensor data is sent from the
device running Sensor Bridge to the device running Sensor Display, where it is then displayed.

The sensor core library is used by both Sensor Bridge and Sensor Display. It provides the following functionality:

* Rendering sensor data to an raster image
* Shared data structures for sensor data
* Shared functionality

## Building and Running the Project

To run the project, execute the following command in the root directory of the project:

```shell
cargo build
```
