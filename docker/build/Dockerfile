# Dockerfile
#
# Development environment based on phusion/baseimage (Ubuntu)
#
# @author      Fabian Haupt <fabian@fabianhaupt.de
# @copyright   2019 Fabian Haupt
# @license     MIT (see LICENSE)
# ------------------------------------------------------------------------------

FROM ubuntu:latest
MAINTAINER fabian@fabianhaupt.de
ENV DEBIAN_FRONTEND noninteractive
ENV TERM linux
ENV HOME /root
ENV DISPLAY :0
# Add SSH keys
ADD id_rsa /home/go/.ssh/id_rsa
ADD id_rsa.pub /home/go/.ssh/id_rsa.pub
RUN echo 'debconf debconf/frontend select Noninteractive' | debconf-set-selections \
# Configure SSH
&& mkdir -p /root/.ssh \
&& echo "Host *" >> /root/.ssh/config \
&& echo "    StrictHostKeyChecking no" >> /root/.ssh/config \
&& echo "    GlobalKnownHostsFile  /dev/null" >> /root/.ssh/config \
&& echo "    UserKnownHostsFile    /dev/null" >> /root/.ssh/config \
&& chmod 600 /home/go/.ssh/id_rsa \
&& chmod 644 /home/go/.ssh/id_rsa.pub \
# Configure default git user
&& echo "[user]" >> /home/go/.gitconfig \
&& echo "	email = gocd@example.com" >> /home/go/.gitconfig \
&& echo "	name = gocd" >> /home/go/.gitconfig \
# Add i386 architecture
# Add repositories and update
&& apt-get update && apt-get -y dist-upgrade \
&& apt-get install -y apt-utils software-properties-common \
&& apt-add-repository universe \
&& apt-add-repository multiverse \
&& apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 3FA7E0328081BFF6A14DA29AA6A19B38D3D831EF \
&& apt-get update \
# Repository for node
&& curl -sL https://deb.nodesource.com/setup_12.x | bash - \
# Set Locale
&& apt-get install -y language-pack-en-base \
&& locale-gen en_US en_US.UTF-8 \
&& dpkg-reconfigure locales \
# install development packages and debugging tools
&& apt-get install -y \
build-essential \
libevent-dev \
libsdl-image1.2-dev \
libsdl-gfx1.2-dev \
libluabind-dev \
ncurses-dev \
libsdl-sge-dev \
# Cleanup temporary data and cache
&& apt-get clean \
&& apt-get autoclean \
&& apt-get -y autoremove \
&& rm -rf /root/.npm/cache/* \
&& rm -rf /root/.composer/cache/* \
&& rm -rf /var/lib/apt/lists/* /tmp/* /var/tmp/*