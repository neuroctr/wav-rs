FROM ubuntu:24.10
RUN apt update && apt upgrade -y \
  && apt install -y curl wget vim git build-essential \
  && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh \
  && source .profile \
  && mkdir ~/repos && cd ~/repos \
  && git clone git@github.com:neuroctr/wav-rs.git
  
