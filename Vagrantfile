Vagrant.configure(2) do |config|
  config.vm.box = "ubuntu/trusty64"

  config.vm.provider "virtualbox" do |vb|
    vb.memory = "4096"
    vb.cpus = 8
  end

  config.vm.provision "shell", inline: <<-SHELL
    sudo apt-get update
    sudo apt-get install -y git clang cmake libssl-dev

    git clone https://github.com/rust-lang/rust.git
    cd rust
    ./configure --prefix=/rust --enable-clang --disable-libcpp --enable-optimize
    make
    cd ..

    export LD_LIBRARY_PATH=$LD_LIBRARY_PATH:/home/vagrant/rust/x86_64-unknown-linux-gnu/stage2/lib/

    git clone https://github.com/rust-lang/cargo
    cd cargo
    ./configure --local-rust-root=../rust/x86_64-unknown-linux-gnu/stage2/ --enable -optimize

    echo "PATH=$PATH:/home/vagrant/cargo/target/snapshot/cargo/bin/:/home/vagrant/rust/x86_64-unknown-linux-gnu/stage2/bin" > ~/.bashrc
    exec bash
  SHELL
end
