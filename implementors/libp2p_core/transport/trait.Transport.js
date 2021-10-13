(function() {var implementors = {};
implementors["libp2p_core"] = [];
implementors["libp2p_dns"] = [{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"libp2p_core/transport/trait.Transport.html\" title=\"trait libp2p_core::transport::Transport\">Transport</a> for <a class=\"struct\" href=\"libp2p_dns/struct.DnsConfig.html\" title=\"struct libp2p_dns::DnsConfig\">DnsConfig</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"libp2p_core/transport/trait.Transport.html\" title=\"trait libp2p_core::transport::Transport\">Transport</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"libp2p_core/transport/trait.Transport.html#associatedtype.Error\" title=\"type libp2p_core::transport::Transport::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"libp2p_core/transport/trait.Transport.html#associatedtype.Dial\" title=\"type libp2p_core::transport::Transport::Dial\">Dial</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a>,&nbsp;</span>","synthetic":false,"types":["libp2p_dns::DnsConfig"]}];
implementors["libp2p_tcp"] = [{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"libp2p_core/transport/trait.Transport.html\" title=\"trait libp2p_core::transport::Transport\">Transport</a> for <a class=\"struct\" href=\"libp2p_tcp/struct.GenTcpConfig.html\" title=\"struct libp2p_tcp::GenTcpConfig\">GenTcpConfig</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: Provider + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::Listener: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::IfWatcher: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::Stream: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a>,&nbsp;</span>","synthetic":false,"types":["libp2p_tcp::GenTcpConfig"]}];
implementors["libp2p_uds"] = [{"text":"impl <a class=\"trait\" href=\"libp2p_core/transport/trait.Transport.html\" title=\"trait libp2p_core::transport::Transport\">Transport</a> for <a class=\"struct\" href=\"libp2p_uds/struct.UdsConfig.html\" title=\"struct libp2p_uds::UdsConfig\">UdsConfig</a>","synthetic":false,"types":["libp2p_uds::UdsConfig"]}];
implementors["libp2p_wasm_ext"] = [{"text":"impl <a class=\"trait\" href=\"libp2p_core/transport/trait.Transport.html\" title=\"trait libp2p_core::transport::Transport\">Transport</a> for <a class=\"struct\" href=\"libp2p_wasm_ext/struct.ExtTransport.html\" title=\"struct libp2p_wasm_ext::ExtTransport\">ExtTransport</a>","synthetic":false,"types":["libp2p_wasm_ext::ExtTransport"]}];
implementors["libp2p_websocket"] = [{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"libp2p_core/transport/trait.Transport.html\" title=\"trait libp2p_core::transport::Transport\">Transport</a> for <a class=\"struct\" href=\"libp2p_websocket/framed/struct.WsConfig.html\" title=\"struct libp2p_websocket::framed::WsConfig\">WsConfig</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"libp2p_core/transport/trait.Transport.html\" title=\"trait libp2p_core::transport::Transport\">Transport</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"libp2p_core/transport/trait.Transport.html#associatedtype.Error\" title=\"type libp2p_core::transport::Transport::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"libp2p_core/transport/trait.Transport.html#associatedtype.Dial\" title=\"type libp2p_core::transport::Transport::Dial\">Dial</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"libp2p_core/transport/trait.Transport.html#associatedtype.Listener\" title=\"type libp2p_core::transport::Transport::Listener\">Listener</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"libp2p_core/transport/trait.Transport.html#associatedtype.ListenerUpgrade\" title=\"type libp2p_core::transport::Transport::ListenerUpgrade\">ListenerUpgrade</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"libp2p_core/transport/trait.Transport.html#associatedtype.Output\" title=\"type libp2p_core::transport::Transport::Output\">Output</a>: <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncRead.html\" title=\"trait futures_io::if_std::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncWrite.html\" title=\"trait futures_io::if_std::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,&nbsp;</span>","synthetic":false,"types":["libp2p_websocket::framed::WsConfig"]},{"text":"impl&lt;T&gt; <a class=\"trait\" href=\"libp2p_core/transport/trait.Transport.html\" title=\"trait libp2p_core::transport::Transport\">Transport</a> for <a class=\"struct\" href=\"libp2p_websocket/struct.WsConfig.html\" title=\"struct libp2p_websocket::WsConfig\">WsConfig</a>&lt;T&gt; <span class=\"where fmt-newline\">where<br>&nbsp;&nbsp;&nbsp;&nbsp;T: <a class=\"trait\" href=\"libp2p_core/transport/trait.Transport.html\" title=\"trait libp2p_core::transport::Transport\">Transport</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/clone/trait.Clone.html\" title=\"trait core::clone::Clone\">Clone</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"libp2p_core/transport/trait.Transport.html#associatedtype.Error\" title=\"type libp2p_core::transport::Transport::Error\">Error</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"libp2p_core/transport/trait.Transport.html#associatedtype.Dial\" title=\"type libp2p_core::transport::Transport::Dial\">Dial</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"libp2p_core/transport/trait.Transport.html#associatedtype.Listener\" title=\"type libp2p_core::transport::Transport::Listener\">Listener</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"libp2p_core/transport/trait.Transport.html#associatedtype.ListenerUpgrade\" title=\"type libp2p_core::transport::Transport::ListenerUpgrade\">ListenerUpgrade</a>: <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,<br>&nbsp;&nbsp;&nbsp;&nbsp;T::<a class=\"type\" href=\"libp2p_core/transport/trait.Transport.html#associatedtype.Output\" title=\"type libp2p_core::transport::Transport::Output\">Output</a>: <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncRead.html\" title=\"trait futures_io::if_std::AsyncRead\">AsyncRead</a> + <a class=\"trait\" href=\"futures_io/if_std/trait.AsyncWrite.html\" title=\"trait futures_io::if_std::AsyncWrite\">AsyncWrite</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Unpin.html\" title=\"trait core::marker::Unpin\">Unpin</a> + <a class=\"trait\" href=\"https://doc.rust-lang.org/1.55.0/core/marker/trait.Send.html\" title=\"trait core::marker::Send\">Send</a> + 'static,&nbsp;</span>","synthetic":false,"types":["libp2p_websocket::WsConfig"]}];
if (window.register_implementors) {window.register_implementors(implementors);} else {window.pending_implementors = implementors;}})()