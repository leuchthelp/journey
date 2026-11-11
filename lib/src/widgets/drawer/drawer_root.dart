import 'package:flutter_hooks/flutter_hooks.dart';
import 'package:hooks_riverpod/hooks_riverpod.dart';
import 'package:material_ui/material_ui.dart';

abstract class JourneyDrawerState extends StatelessWidget {
  const new({super.key, this.child, required this.onPressed});
  final Widget? child;
  final VoidCallback onPressed;
}

class JourneyDrawerClosed extends JourneyDrawerState {
  const new({super.key, required super.onPressed});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Spacer(),
        TextButton(onPressed: onPressed, child: Text("settings")),
      ],
    );
  }
}

class JourneyDrawerOpen extends JourneyDrawerState {
  const new({super.key, super.child, required super.onPressed});

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Column(
          children: [
            Spacer(),
            TextButton(onPressed: onPressed, child: Text("settings")),
          ],
        ),
        ?child,
      ],
    );
  }
}

class JourneyDrawerRoot extends HookConsumerWidget {
  const JourneyDrawerRoot({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final opened = useState(false);

    return opened.value
        ? JourneyDrawerOpen(
            onPressed: () => {opened.value = !opened.value},
            child: Column(
              children: [
                TextButton(onPressed: () => {}, child: Text("Providers")),
              ],
            ),
          )
        : JourneyDrawerClosed(onPressed: () => {opened.value = !opened.value});
  }
}
