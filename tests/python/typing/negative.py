import btpc

btpc.CreateOptions(trackers=((b"bytes-not-text",),))
btpc.CreateOptions(nodes=(("host", "not-a-port"),))
btpc.Metainfo.from_bytes(b"de").edit(comment=b"bytes-not-text")
