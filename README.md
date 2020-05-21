# Laurn

Run a dev-environment in a pure-ish nix environment.

Laurn will read your `laurn.nix` file and when running `laurn shell` you will get your project directory mounted in a namespace where only your project directory and your declared dependencies are available.
The purpose is to isolate your system from your developement environment:
 - Dependencies declaration is "pure", what's not declared is not available.
 - No libraries can extract secrets from your host (npm tokens, ssh keys, ...).

## Usage

`laurn shell`

output:
```
bash-4.4$ mount
/dev/mapper/vic-root on /nix/store/0rah6c2bqy6lqwh9z261nc2wd5lhaxya-ncurses-6.1-20190112-man type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/mxaxvp33wg9sim8qh2kkw041v492bvxj-libunistring-0.9.10 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/hjng28vbd73qq9iz9j8r397x19aa8fp9-libidn2-2.3.0 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/8g1v3sf0xvf044sz1b4kcrg1i86z9bh7-glibc-2.30 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/18g0mnrxq829yw1fdn6lr92icsi97irb-ncurses-6.1-20190112 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/s80jmwnnr5sa5hvxkdz44ksvbng1rycc-zlib-1.2.11 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/vrnxw026hiy8jvdpaaix47x528bkzksk-bash-4.4-p23 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/1d7j21yxvgg5s6nbynnysnk0f2q1w6ax-cracklib-2.9.7 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/1p4pfi7ji7gvlpgb6g6f7j9iyiddnxsy-libffi-3.3 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/34zpi8brqj78h842ski7swaj7psrl85b-gcc-9.2.0-lib type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/1pp034ghcsb09nfnc586g0f9iqxq1zla-db-4.8.30 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/1z5b7fjgsh9hg397wjg21f2hnv5rygjs-readline-7.0p5 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/6hhxlbabm7xxdfdw9mbxb8llh63k1bxx-linux-headers-4.19.16 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/hi6v8iga1pliafmcsaw3dp90lnmndii6-glibc-2.30-bin type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/2v6pi2wj3lcsc3j48n7flx9mgqyii1lv-glibc-2.30-dev type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/khdvzbn3jlq5mzybxmkgi1x2j2n9ncg4-keyutils-1.6.1-lib type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/34xkmlqqqwzqncd4d5d8njm68xwln5k8-libkrb5-1.17 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/3qwd41p2cwam035r3ik62ri4dcrgmlyg-attr-2.4.48 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/48ww0x7kxsh4s28ccn158f4251xjwaaz-expand-response-params type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/4f3xb633flsm1lina69z4nd8yj9zxiz7-lz4-1.9.1 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/6ws1wa6ak9p4qmdfcycfwalqh29hds05-nghttp2-1.40.0-lib type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/814wbxsa1xwbc9dba504ryfqaykzqvb4-openssl-1.1.1d type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/851x6m6dzzzixca7a7w9fpdgrr2zh3yn-libssh2-1.9.0 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/5na95ifvbvivp961argc3fhayldd4m3l-curl-7.68.0 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/b5k58xdn5gw84r23vxasbkfaqinmpld9-pcre-8.43 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/7lc9vbmdxp7kjqchiqsd2g89r6pc43w7-gnugrep-3.4 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/d8d2x7q98yzkcdnllk784rkmh70ynn91-nss-cacert-3.49.2 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/p9v6d6141ymlp4n9hywp52sh3f1cy2ld-libxml2-2.9.10 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/bnqklx5bps0inky5445nc42142ih79np-llvm-9.0.1-lib type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/hxpyfbmwj3w74653i21gbbaqcfvvpnqn-ncurses-6.1-20190112-dev type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/q58ps0cs3v2xicfh26wnvzgvabvvp530-libpfm-4.10.1 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/xhs17wkshs8a8ws6s5lj4kh20yvzh08a-zlib-1.2.11-dev type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/lhn4n1xwlqawp7gzj5lhqs1v7jqk8khx-llvm-9.0.1 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/y57skwl8a5vbkrjrc30ygdw9vr1p6n19-gcc-9.2.0 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/y41n9f36mhvsv6jr1fxfxgl781lhcz15-rustc-1.41.0 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/7v2q1yrixn7qb7gbscxcqn2xw36mlaxy-cargo-1.41.0 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/jy73kzgc92zgnnxcdrdhckjjkf3wmzdr-xz-5.2.4 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/998ca81cb0vjbc5xhycncginlyr9nj42-libunwind-1.3.1 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/99azsj9swsvj721n3mwxn61ww5v464lx-libgpg-error-1.36 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/l67zr4vmyc90y7xw31kfbr5r794ln96q-linux-pam-1.3.1 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/k049sqrbm1rzx7rilqsr3kr3jhvfncsl-libcap-2.27-lib type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/r6hbqfvwis5qjs9vppgmdcmsmh0hmf2k-libgcrypt-1.8.5 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/lz31apmyh255xq5z7kzrxihss9gv0ngx-systemd-243.4-lib type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/nkxkjka1xpzp75qnyzq94i9gg7i5fyhs-shadow-4.8 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/w8r4z03c007l0xwhdphfh72q5x47ywjs-util-linux-2.33.2 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/qnd24l8bqc9ph7c0ir8v5m6fd8ha7395-util-linux-2.33.2-bin type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/b55p7s4gpaqpsqqvji632m05z0p8044n-mount-1003.1-2008 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/bbr22zfd1z9s4riy5cf1f0xkx2596wmr-libnfnetlink-1.0.1 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/g3vkfhbh0j5dg2i0ja4rz6bspkb46k3r-acl-2.2.53 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/scgxalx94hbc60px2bazzap0n3n3w897-coreutils-8.31 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/xpzdkp9y8fnliwrs2g5pygx2qmmbrv8w-binutils-2.31.1 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/clxsd876533bwjlmfza5r5m0pj80297a-binutils-wrapper-2.31.1 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/cp3a6dkc0i9hqk6hx3ydm5j4wsh0mi8z-procps-3.3.16 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/ssg4ghwc1n1n2hc6yn5q9s32m0l4yws3-libmnl-1.0.4 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/dipp7rz6v4cj89vs1f58w408r231vj7n-libnftnl-1.1.5 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/f881lww9f0svpya9w3rqqy7v4s9ihd7x-libnetfilter_conntrack-1.0.7 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/fqhjxf9ii4w4gqcsx59fyw2vvj91486a-gcc-wrapper-9.2.0 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/h5z0qskvgq0iwiw176g9n04zp1q98k5w-which-2.21 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/i7czk5aswy1nzh708rci1rq1l9dcljy4-strace-5.5 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/jdmgg1biizi6mn4m202p3hbhzr3rnil2-libpcap-1.9.1 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/lnx06v8j2wwc56zr6wszr05xf6z3nhlq-db-5.3.28 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/xipx492glagsy1pn30594rd6qvd7g2x9-libelf-0.8.13 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/zjz7nh2rmndng304dfqfc92kxhb85d3a-iptables-1.8.4 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/n3l9xmwmm1swsfq4psvdwban4zg62adx-iproute2-5.5.0 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/v9i377m22afk3xybpwbq50yz29jark1r-bash-interactive-4.4-p23 type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /nix/store/kn3hd5a49vb7l280n94h5jy92314syc8-laurn-shell type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-root on /etc/resolv.conf type ext4 (ro,relatime,errors=remount-ro)
/dev/mapper/vic-home on /home/baloo/dev/laurn type ext4 (rw,relatime)
/dev/mapper/vic-home on /home/baloo/dev/laurn/.git type ext4 (ro,relatime)
/dev/mapper/vic-home on /home/baloo/dev/laurn/.laurnrc type ext4 (ro,relatime)
/dev/mapper/vic-home on /home/baloo/dev/laurn/laurn.nix type ext4 (ro,relatime)
/dev/mapper/vic-home on /home/baloo/dev/laurn/nix type ext4 (ro,relatime)
/dev/mapper/vic-home on /home/baloo/.cargo type ext4 (rw,relatime)
udev on /dev/null type devtmpfs (rw,nosuid,relatime,size=16380964k,nr_inodes=4095241,mode=755)
udev on /dev/console type devtmpfs (rw,nosuid,relatime,size=16380964k,nr_inodes=4095241,mode=755)
udev on /dev/random type devtmpfs (rw,nosuid,relatime,size=16380964k,nr_inodes=4095241,mode=755)
udev on /dev/urandom type devtmpfs (rw,nosuid,relatime,size=16380964k,nr_inodes=4095241,mode=755)
udev on /dev/tty type devtmpfs (rw,nosuid,relatime,size=16380964k,nr_inodes=4095241,mode=755)
udev on /dev/zero type devtmpfs (rw,nosuid,relatime,size=16380964k,nr_inodes=4095241,mode=755)
proc on /proc type proc (rw,nosuid,nodev,noexec,relatime)
```


## Shell hook

```
eval "$(cargo run hook bash)
```


# Limitations

laurn requires:

  - a recent linux kernel (with user namespaces)
  - nix
