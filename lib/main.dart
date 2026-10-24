import 'package:material_ui/material_ui.dart';
import 'package:journey/src/rust/api/simple.dart';
import 'package:journey/src/rust/frb_generated.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(const Journey());
}

class Journey extends StatelessWidget {
  const Journey({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Scaffold(
        appBar: AppBar(title: const Text('flutter_rust_bridge quickstart')),
        body: Center(
          child: Text(
            'Action: Call Rust `greet("Tom")`\nResult: `${greet(name: "Tom")}`',
          ),
        ),
      ),
    );
  }
}
