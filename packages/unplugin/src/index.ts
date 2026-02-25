import { createUnplugin } from 'unplugin';

export const rastUnplugin = createUnplugin(() => {
  return {
    name: 'rast-unplugin',
    enforce: 'post',
    transform(code, id) {
      // Placeholder - will be implemented in Task 4 with bindings integration
      if (id.endsWith('.ts') || id.endsWith('.js')) {
        return { code };
      }
    },
  };
});

export default rastUnplugin;
