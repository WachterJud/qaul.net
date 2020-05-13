Build qaul.net release-1.0.0 on Raspberry Pi 2 running Ubuntu Mate
==================================================================

Prerequesites
-------------

Install needed software to download and build qaul.net from source.

	sudo apt-get install git cmake build-essential pkg-config \
	libgtk-3-dev  libwebkit2gtk-4.0-dev libdbus-1-dev autotools-dev \
	libasound2-dev bison flex automake


Download and Build
------------------

	# Download the source from https://github.com/qaul/qaul.net, checkout 
    # branch release-1.0.0, download and init submodules
	git clone --recursive --branch release-1.0.0 https://github.com/qaul/qaul.net.git

	# create build directory
	cd qaul.net
	mkdir build
	cd build
	
	# generate make files
	cmake .. -DRaspberry=1 -DCMAKE_INSTALL_PREFIX="/opt/qaul"
	
	# make and install qaul.net
	make
	sudo make install


Run qaul.net GUI client
-----------------------

Run qaul.net GKT client from the command line

	# run qaul.net GUI client form the command line	
	/opt/qaul/bin/qaul-gtk


Link the binary to your execution path

	sudo ln -s /opt/qaul/bin/qaul-gtk /usr/local/bin/qaul-gtk


Create the .deb Installer package
---------------------------------

To create the `.deb` installer, execute the following command in your build 
directory.

	# create the installer package
	make package
