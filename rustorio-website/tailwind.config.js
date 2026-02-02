module.exports = {
  corePlugins: {
    preflight: true, // keep this true
  },
  theme: {
    extend: {},
  },
  plugins: [
    function({ addBase }) {
      addBase({
        'img, svg, video, canvas, audio, iframe, embed, object': {
          'display': 'block',
        },
      })
    },
  ],
}