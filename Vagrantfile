Vagrant.configure(2) do |config|
  config.vm.box = "ubuntu/trusty64"

  config.vm.provider "virtualbox" do |vb|
    vb.memory = "4096"
    vb.cpus = 8
  end

  config.vm.provision "shell", inline: <<-SHELL
    sudo apt-get update
    sudo apt-get install -y git clang cmake

    git clone https://github.com/rust-lang/rust.git
    cd rust
    ./configure --prefix=/rust --enable-clang --disable-libcpp --enable-optimize
    make
    cd ..

    git clone https://github.com/rust-lang/cargo
    cd cargo
    ./configure
  SHELL
end
