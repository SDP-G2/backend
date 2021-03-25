# install: deps docker compose src

# deps:
sudo apt install git python curl postgresql
sudo systemctl stop postgresql
# docker:
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $(USER)
sudo service docker start
# compose:
sudo curl -L "https://github.com/docker/compose/releases/download/1.28.5/docker-compose-`uname -s`-`uname -m`" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
# src:
git clone https://github.com/SDP-G2/backend
