import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:journey/src/widgets/drawer/drawer_root.dart';
import 'package:journey/src/rust/frb_generated.dart';
import 'package:material_ui/material_ui.dart';

Future<void> main() async {
  await RustLib.init();
  runApp(ProviderScope(child: const Journey()));
}

class Journey extends StatelessWidget {
  const Journey({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      home: Container(
        padding: EdgeInsetsGeometry.fromLTRB(10, 0, 0, 10),
        color: const Color.from(
          alpha: 1,
          red: 0.906,
          green: 0.871,
          blue: 0.757,
        ),
        child: Row(children: [JourneyDrawerRoot(), Spacer(flex: 1)]),
      ),
    );
  }
}
