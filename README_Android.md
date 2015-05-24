Android Instructions
====================

This page contains all the instructions on building Qual.net on the Android platform.

Compatibility
-------------

Qaul.net was tested on the following devices:

| Device            | 2.3 | 4.x | CM9 |
| ----------------- | --- | --- | --- |
| Samsung Galaxy S2 | Y   | N   | N   |
| HTC Sensation     | ?   | Y   | Y   |
| fairphone         | ?   | Y   | ?   |

Legend:

* Y = it works
* N = it doesn't work yet
* ? = it wasn't tested yet


Problems & Solutions:

* qaul.net needs the IBSS/Ad-hoc wifi mode. This Wifi-Standard mode was 
  removed by google. 
  * On Android < 4.x IBSS mode can be activated via the wext (wireless 
    extensions). 
    Some vendors ship also their 4.x devices with wext.
  * On Android >= 4 IBSS mode can be activated on some devices via 
    wpa_supplicant.
* qaul.net needs ARM and Android > 2.3 Gingerbread
  (due to before unsupported pthread functions in pjsip). 
* The Android device needs to be rooted.


Prerequisites
-------------

Below are all the required tools and files you will need in order to build Qual.net.

### Android SDK/NDK

For building the Android version of Qaul.net you will need both the Source
Develpment Kit *and* the Native Development Kit

get NDK https://developer.android.com/tools/sdk/ndk/index.html

get SDK https://developer.android.com/sdk/index.html
add sdk to path if not done by installer

### Needed extra libraries

create directory for extra libs

cd into it

	adb pull /system/lib/libcutils.so
	adb pull /system/lib/libwpa_client.so
	adb pull /system/bin/ifconfig
	adb pull /system/bin/iptables


Get the source
--------------

Download the source from <insert source here> or
get the sources from github: git clone https://github.com/WachterJud/qaul.net.git


Build
-----

    mkdir builddir
    cd builddir

    cmake ../path/to/source -DPORT=Android -DNDK_ROOT=/path/to/ndk -DEXTRALIB_PATH=/path/to/extra/lib
    make

find the apk in android/bin


Eclipse
-------

- Import Existing Project
- Do **NOT** copy files.

