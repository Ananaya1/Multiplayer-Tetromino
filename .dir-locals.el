; Rust analyzer configuration
((rust-mode . ((eglot-workspace-configuration
                . (:rust-analyzer
                   (
                    :linkedProjects [
                                     "client/Cargo.toml"
                                     "common/Cargo.toml"
                                     "server/Cargo.toml"
                                     ]
                    :rustfmt (:extraArgs ["--edition" "2024"])
                    :cargo (:target nil)
                           )))))
 ; (rust-ts-mode . ((eglot-workspace-configuration
 ;                   . ("rust-analyzer"
 ;                      (:cargo (:target "wasm32-unknown-unknown")
 ;                              :rustfmt (:extraArgs ["--edition" "2024"]))))))
 )
